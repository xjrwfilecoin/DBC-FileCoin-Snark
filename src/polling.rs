use std::collections::HashMap;
use std::os::unix::thread::JoinHandleExt;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, TryRecvError};
use std::thread::JoinHandle;

use libc::pthread_cancel;
use log::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use lazy_static::lazy_static;

lazy_static! {
    static ref WORKER_TOKEN: AtomicU64 = AtomicU64::new(0);
    static ref WORKER_INIT: AtomicBool = AtomicBool::new(false);
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum PollingState {
    Started(u64),
    Pending,
    Done(Value),
    Removed,
    Error(PollingError),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum PollingError {
    NotExist,
    Disconnected,
}

pub struct WorkerProp {
    handle: JoinHandle<()>,
    receiver: Receiver<Value>,
}

impl WorkerProp {
    pub fn new(handle: JoinHandle<()>, receiver: Receiver<Value>) -> Self {
        Self { handle, receiver }
    }
}

pub struct ServState {
    workers: HashMap<u64, WorkerProp>,
}

impl ServState {
    pub fn new() -> Self {
        // NOTE: ensure ServState is init only once
        assert_eq!(WORKER_INIT.swap(true, Ordering::SeqCst), false);

        Self {
            workers: HashMap::new(),
        }
    }

    pub fn enqueue(&mut self, prop: WorkerProp) -> PollingState {
        let token = WORKER_TOKEN.fetch_add(1, Ordering::SeqCst);
        self.workers.insert(token, prop);

        PollingState::Started(token)
    }

    pub fn get(&mut self, token: u64) -> PollingState {
        let state = self
            .workers
            .get(&token)
            .map(|x| match x.receiver.try_recv() {
                Ok(r) => PollingState::Done(r),
                Err(TryRecvError::Empty) => PollingState::Pending,
                Err(TryRecvError::Disconnected) => PollingState::Error(PollingError::Disconnected),
            })
            .unwrap_or(PollingState::Error(PollingError::NotExist));

        match &state {
            PollingState::Done(_) => {
                debug!("Job {} removed dut to finish", token);
                self.workers.remove(&token);
            }
            _ => {}
        };

        state
    }

    pub fn remove(&mut self, token: u64) -> PollingState {
        if let Some(prop) = self.workers.remove(&token) {
            debug!("Job {} force removed", token);
            let pthread_t = prop.handle.into_pthread_t();

            unsafe {
                pthread_cancel(pthread_t);
            }

            return PollingState::Removed;
        }

        PollingState::Error(PollingError::NotExist)
    }
}

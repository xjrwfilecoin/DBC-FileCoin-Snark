use crate::polling::*;
use actix_multipart::Multipart;
use actix_web::web::{self, Data, Json};
use actix_web::{Error, HttpResponse};
use futures::stream::{StreamExt, TryStreamExt};
use log::trace;
use serde::Serialize;
use serde_json::json;
use std::io::Write;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

#[derive(Serialize)]
struct ServerLoad {
    limit: u64,
    current: u64,
}

pub async fn test() -> HttpResponse {
    trace!("test");

    HttpResponse::Ok().body("Worked!")
}

pub async fn test_polling(state: Data<Arc<Mutex<ServState>>>) -> HttpResponse {
    trace!("test polling");

    let (tx, rx) = channel();
    let handle: JoinHandle<()> = thread::spawn(move || {
        thread::sleep(Duration::from_secs(30));
        let r = "Ok!!!";

        tx.send(json!(r)).unwrap();
    });

    let prop = WorkerProp::new("Test".to_string(), handle, rx);
    let response = state.lock().unwrap().enqueue(prop);
    HttpResponse::Ok().json(response)
}

pub async fn query_load(state: Data<Arc<Mutex<ServState>>>, phase: Json<String>) -> HttpResponse {
    trace!("query_load: {:?}", phase);

    let current = state.lock().unwrap().job_num(&phase.0);

    let data = ServerLoad { limit: 5, current };

    HttpResponse::Ok().json(data)
}

pub async fn query_state(state: Data<Arc<Mutex<ServState>>>, token: Json<u64>) -> HttpResponse {
    trace!("query_state: {:?}", token);

    let response = state.lock().unwrap().get(*token);

    HttpResponse::Ok().json(response)
}

pub async fn remove_job(state: Data<Arc<Mutex<ServState>>>, token: Json<u64>) -> HttpResponse {
    trace!("remove_job: {:?}", token);

    let response = state.lock().unwrap().remove(*token);

    HttpResponse::Ok().json(response)
}

pub async fn upload_file(mut payload: Multipart) -> Result<HttpResponse, Error> {
    trace!("upload_file");

    let mut ret_path: Option<String> = None;

    // iterate over multipart stream
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_type = field.content_disposition().unwrap();
        let filename = content_type.get_filename().unwrap();
        let filepath = format!("/tmp/upload/{}", filename);
        trace!("got file: {}", filepath);
        ret_path = Some(filepath.clone());

        // File::create is blocking operation, use threadpool
        let mut f = web::block(|| std::fs::File::create(filepath)).await.unwrap();

        // Field in turn is stream of *Bytes* object
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            // filesystem operations are blocking, we have to use threadpool
            f = web::block(move || f.write_all(&data).map(|_| f)).await?;
        }
    }

    // TODO: file name
    Ok(HttpResponse::Ok().json(ret_path))
}

pub async fn upload_test() -> HttpResponse {
    let html = r#"<html>
        <head><title>Upload Test</title></head>
        <body>
            <form action="/sys/upload_file" target="/sys/upload_file" method="post" enctype="multipart/form-data">
                <input type="file" multiple name="file"/>
                <input type="submit" value="Submit">
            </form>
        </body>
	    </html>"#;

    HttpResponse::Ok().body(html)
}

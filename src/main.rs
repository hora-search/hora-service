use actix_web::{post, web, App, Error, HttpResponse, HttpServer, Responder};
use futures_util::StreamExt;
use real_hora::core::ann_index;
use real_hora::core::metrics;
use serde::Deserialize;
use std::collections::HashMap;
use std::iter::Iterator;
use std::sync::Mutex;
#[macro_use]
extern crate lazy_static;

trait ANNIndex: ann_index::ANNIndex<f32, usize> + ann_index::SerializableIndex<f32, usize> {}

lazy_static! {
    static ref ANNIndexMnger: Mutex<HashMap<String, Box<dyn real_hora::core::ann_index::ANNIndex<f32, usize>>>> =
        { Mutex::new(HashMap::new()) };
}

pub fn metrics_transform(s: &str) -> metrics::Metric {
    match s {
        "angular" => metrics::Metric::Angular,
        "manhattan" => metrics::Metric::Manhattan,
        "dot_product" => metrics::Metric::DotProduct,
        "euclidean" => metrics::Metric::Euclidean,
        "cosine_similarity" => metrics::Metric::CosineSimilarity,
        _ => metrics::Metric::Unknown,
    }
}

#[derive(Deserialize)]
struct AddItem {
    features: Vec<f32>,
    idx: usize,
}

#[post("/new/{index_type}")]
async fn new(
    web::Path(_index_type): web::Path<String>,
    mut payload: web::Payload,
    _data: web::Data<Mutex<HashMap<String, Box<dyn ann_index::ANNIndex<f32, usize>>>>>,
) -> Result<HttpResponse, Error> {
    let mut bytes = web::BytesMut::new();
    while let Some(item) = payload.next().await {
        bytes.extend_from_slice(&item?);
    }

    ANNIndexMnger.lock().unwrap();

    // match index_type {
    //     "hnsw_index" => {
    //         let v = serde_json::from_slice<hora::index::HNSWParams>(&body)?;
    //         data
    //     }
    // }

    Ok(HttpResponse::Ok().finish())
}

#[post("/add/{index_name}")]
async fn add(
    _path: web::Path<String>,
    _json: web::Json<AddItem>,
    _data: web::Data<Mutex<HashMap<String, Box<dyn ann_index::ANNIndex<f32, usize>>>>>,
) -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || App::new().service(new).service(add))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}

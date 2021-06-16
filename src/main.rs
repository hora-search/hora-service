use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use real_hora::core::ann_index;
use real_hora::core::metrics;
use std::sync::Mutex;
use std::collections::HashMap;
use serde::Deserialize;

trait ANNIndex: ann_index::ANNIndex<f32, usize> + ann_index::SerializableIndex<f32, usize> {}

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

static mut ANNIndexManager: Option<Mutex<
    HashMap<String, Box<ann_index::ANNIndex<f32, usize>>>,
>> = None;

#[derive(Deserialize)]
struct AddItem {
    features: Vec<f32>,
    idx: usize,
}

#[get("/new")]
async fn new() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/add")]
async fn add(path: web::Path<String>, json: web::Json<AddItem>) -> impl Responder {

    HttpResponse::Ok().body("Hello world!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    ANNIndexManager = Some(Mutex::new(HashMap::new()));
    HttpServer::new(|| {
        App::new()
            .service(add)
            .service(new)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

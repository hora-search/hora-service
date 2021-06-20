use actix_web::{post, web, App, Error, HttpResponse, HttpServer, Responder, Result};
use futures_util::StreamExt;

use real_hora::core::ann_index;
use real_hora::core::metrics;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::iter::Iterator;
use std::sync::Mutex;
#[macro_use]
extern crate lazy_static;

trait ANNIndex: ann_index::ANNIndex<f32, usize> + ann_index::SerializableIndex<f32, usize> {}

lazy_static! {
    static ref ANN_INDEX_MANAGER: Mutex<HashMap<String, Box<dyn real_hora::core::ann_index::ANNIndex<f32, usize>>>> =
        Mutex::new(HashMap::new());
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
    features: Vec<Vec<f32>>,
    idx: Vec<usize>,
}

#[derive(Deserialize, Serialize)]
struct ResultItem {
    idx: Vec<usize>,
}

#[derive(Deserialize)]
struct SearchItem {
    features: Vec<f32>,
    k: usize,
}

#[derive(Deserialize)]
struct NewItem {
    dimension: usize,
    index_name: String,
}

// TODO: use params inside the json
#[post("/new/{index_type}")]
async fn new(
    web::Path(index_type): web::Path<String>,
    mut payload: web::Payload,
    index_info: web::Query<NewItem>,
) -> Result<HttpResponse, Error> {
    let mut bytes = web::BytesMut::new();
    while let Some(item) = payload.next().await {
        bytes.extend_from_slice(&item?);
    }
    match index_type.as_str() {
        "hnsw_index" => {
            let v =
                serde_json::from_slice::<real_hora::index::hnsw_params::HNSWParams<f32>>(&bytes)?;
            ANN_INDEX_MANAGER.lock().unwrap().insert(
                index_info.index_name.to_string(),
                Box::new(real_hora::index::hnsw_idx::HNSWIndex::new(
                    index_info.dimension,
                    &v,
                )),
            );
        }
        "ssg_index" => {
            let v = serde_json::from_slice::<real_hora::index::ssg_params::SSGParams<f32>>(&bytes)?;
            ANN_INDEX_MANAGER.lock().unwrap().insert(
                index_info.index_name.to_string(),
                Box::new(real_hora::index::ssg_idx::SSGIndex::new(
                    index_info.dimension,
                    &v,
                )),
            );
        }
        "bruteforce_index" => {
            let v = serde_json::from_slice::<real_hora::index::bruteforce_params::BruteForceParams>(
                &bytes,
            )?;
            ANN_INDEX_MANAGER.lock().unwrap().insert(
                index_info.index_name.to_string(),
                Box::new(real_hora::index::bruteforce_idx::BruteForceIndex::new(
                    index_info.dimension,
                    &v,
                )),
            );
        }
        "pq_index" => {
            let v = serde_json::from_slice::<real_hora::index::pq_params::PQParams<f32>>(&bytes)?;
            ANN_INDEX_MANAGER.lock().unwrap().insert(
                index_info.index_name.to_string(),
                Box::new(real_hora::index::pq_idx::PQIndex::new(
                    index_info.dimension,
                    &v,
                )),
            );
        }
        "ivfpq_index" => {
            let v =
                serde_json::from_slice::<real_hora::index::pq_params::IVFPQParams<f32>>(&bytes)?;
            ANN_INDEX_MANAGER.lock().unwrap().insert(
                index_info.index_name.to_string(),
                Box::new(real_hora::index::pq_idx::IVFPQIndex::new(
                    index_info.dimension,
                    &v,
                )),
            );
        }
        _ => {
            return Ok(HttpResponse::NotFound().finish());
        }
    }

    Ok(HttpResponse::Ok().finish())
}

#[post("/add/{index_name}")]
async fn add(path: web::Path<String>, json: web::Json<AddItem>) -> impl Responder {
    assert_eq!(json.features.len(), json.idx.len());
    json.features
        .iter()
        .zip(json.idx.iter().copied())
        .for_each(|(i, f)| {
            ANN_INDEX_MANAGER
                .lock()
                .unwrap()
                .get_mut(&path.to_string())
                .unwrap()
                .add(i, f)
                .unwrap();
        });

    HttpResponse::Ok().finish()
}

#[post("/build/{index_name}")]
async fn build(path: web::Path<String>, mt: web::Query<String>) -> impl Responder {
    match ANN_INDEX_MANAGER.lock().unwrap().get_mut(&path.to_string()) {
        Some(idx) => idx.build(metrics_transform(&mt)).unwrap(),
        None => return HttpResponse::NotFound().finish(),
    };

    HttpResponse::Ok().finish()
}

#[post("/search/{index_name}")]
async fn search(path: web::Path<String>, query: web::Query<SearchItem>) -> Result<HttpResponse> {
    match ANN_INDEX_MANAGER.lock().unwrap().get_mut(&path.to_string()) {
        Some(idx) => Ok(HttpResponse::Ok().json(ResultItem {
            idx: idx.search(&query.features, query.k),
        })),
        None => Ok(HttpResponse::NotFound().finish()),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || App::new().service(new).service(add))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}

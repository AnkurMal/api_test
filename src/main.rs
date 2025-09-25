mod database;
use std::{env, error::Error, net::SocketAddr};

use axum::{
    Json, Router,
    response::IntoResponse,
    routing::{get, post},
};
use futures::TryStreamExt;
use mongodb::bson::{Document, doc};
use serde::Deserialize;
use serde_json::json;
use tower_http::cors::{Any, CorsLayer};

use database::get_database;

#[derive(Deserialize)]
struct Credentials {
    username: String,
    password: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let port: u16 = env::var("PORT").unwrap_or("3000".to_string()).parse()?;
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/posts", get(get_posts))
        .route("/register", post(verify_user))
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("Server running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn verify_user(Json(credentials): Json<Credentials>) -> impl IntoResponse {
    let coll = get_database().await.collection::<Document>("credentials");
    let filter = doc! {
        "username": credentials.username,
        "password": credentials.password
    };

    match coll.find_one(filter).await.unwrap() {
        Some(_) => Json(json!({ "success": true })),
        _ => Json(json!({ "success": false })),
    }
}

async fn get_posts() -> impl IntoResponse {
    let coll = get_database().await.collection::<Document>("posts");

    let mut cursor = coll.find(doc! {}).await.unwrap();
    let mut json_array = Vec::new();

    while let Some(doc) = cursor.try_next().await.unwrap() {
        json_array.push(json!(serde_json::to_value(&doc).unwrap()));
    }

    Json(json_array)
}

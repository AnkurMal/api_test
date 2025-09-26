mod database;
use std::{env, error::Error, net::SocketAddr};

use axum::{
    Json, Router,
    response::IntoResponse,
    routing::{get, post},
};
use database::*;
use futures::TryStreamExt;
use mongodb::bson::{self, Document, doc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tower_http::cors::{Any, CorsLayer};

#[derive(Deserialize, Serialize, Debug)]
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
        .route("/login", post(user_login))
        .route("/register", post(user_register))
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("Server running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn user_login(Json(credentials): Json<Credentials>) -> impl IntoResponse {
    let coll = db!().collection::<Document>("credentials");
    let filter = doc! {
        "username": credentials.username
    };

    match coll.find_one(filter).await.unwrap() {
        Some(res) => {
            let res = bson::from_document::<Credentials>(res).unwrap();
            if credentials.password == res.password {
                Json(json!({ "username": true, "password": true }))
            } else {
                Json(json!({ "username": true, "password": false }))
            }
        }
        _ => Json(json!({ "username": false})),
    }
}

async fn user_register(Json(credentials): Json<Credentials>) -> impl IntoResponse {
    let coll = db!().collection::<Document>("credentials");
    let filter = doc! {
        "username": &credentials.username
    };

    match coll.find_one(filter).await.unwrap() {
        Some(_) => Json(json!({ "success": false})),
        _ => {
            coll.insert_one(bson::to_document(&credentials).unwrap())
                .await
                .unwrap();
            Json(json!({ "success": true}))
        }
    }
}

async fn get_posts() -> impl IntoResponse {
    let coll = db!().collection::<Document>("posts");

    let mut cursor = coll.find(doc! {}).await.unwrap();
    let mut json_array = Vec::new();

    while let Some(doc) = cursor.try_next().await.unwrap() {
        json_array.push(json!(serde_json::to_value(&doc).unwrap()));
    }

    Json(json_array)
}

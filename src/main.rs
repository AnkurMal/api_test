use std::{env, error::Error, net::SocketAddr, sync::Arc};

use axum::{Extension, Json, Router, routing::get};
use futures::TryStreamExt;
use mongodb::{
    Client, Collection,
    bson::{Document, doc},
};
use serde_json::{Value, json};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let port: u16 = env::var("PORT").unwrap_or("3000".to_string()).parse()?;

    let uri = env::var("MONGO_URI").expect("MONGO_URI environment variable must be set");
    let client = Client::with_uri_str(uri).await?;

    let database = client.database("sample");
    let coll = database.collection::<Document>("posts");
    let shared_coll = Arc::new(coll);

    let app = Router::new()
        .route("/posts", get(get_posts))
        .layer(Extension(shared_coll));

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("Server running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn get_posts(Extension(coll): Extension<Arc<Collection<Document>>>) -> Json<Vec<Value>> {
    let mut cursor = coll.find(doc! {}).await.unwrap();
    let mut json_array = Vec::new();

    while let Some(doc) = cursor.try_next().await.unwrap() {
        json_array.push(json!(serde_json::to_value(&doc).unwrap()));
    }

    Json(json_array)
}

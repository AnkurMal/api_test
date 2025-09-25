#![allow(unused)]
use std::env;

use mongodb::{Client, Database};
use tokio::sync::OnceCell;

const DATABASE_NAME: &str = "ticketing";
static DATABASE: OnceCell<Database> = OnceCell::const_new();

pub async fn get_database() -> &'static Database {
    DATABASE
        .get_or_init(|| async {
            let uri = env::var("MONGO_URI").expect("MONGO_URI environment variable must be set");
            let client = Client::with_uri_str(uri)
                .await
                .expect("Failed to create MongoDB client");
            client.database(DATABASE_NAME)
        })
        .await
}

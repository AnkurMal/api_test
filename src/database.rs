#![allow(unused)]
use std::env;

use mongodb::Database;
use tokio::sync::OnceCell;

pub const DATABASE_NAME: &str = "ticketing";
pub static DATABASE: OnceCell<Database> = OnceCell::const_new();

#[macro_export]
macro_rules! db {
    () => {
        DATABASE
            .get_or_init(|| async {
                let uri =
                    env::var("MONGO_URI").expect("MONGO_URI environment variable must be set");
                let client = mongodb::Client::with_uri_str(uri)
                    .await
                    .expect("Failed to create MongoDB client");
                client.database(DATABASE_NAME)
            })
            .await
    };
}

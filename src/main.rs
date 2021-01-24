use std::sync::Arc;

use log::{error, info, warn};
use sha2::{Digest, Sha256};
use sqlx::prelude::*;
use warp::Filter;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::init();

    // e.g. sqlite://mydb.sqlite
    // See: https://docs.rs/sqlx-core/0.4.2/sqlx_core/sqlite/struct.SqliteConnectOptions.html
    info!("Getting db pool");
    let dburl = std::env::var("DATABASE_URL")?;
    let pool = Arc::new(sqlx::sqlite::SqlitePool::connect(&dburl).await?);
    let upload_handler = Arc::new(UploadHandler::new(pool.clone()));
    let fetch_handler = Arc::new(FetchHandler::new(pool.clone()));

    info!("Running migrations");
    {
        let m = sqlx::migrate!();
        m.run(&(*pool)).await?;
    }
    info!("Done running migrations");

    info!("Starting server");
    let upload = warp::path::end()
        .and(warp::put())
        .and(warp::body::content_length_limit(1024 * 1024 * 100))
        .and(warp::body::bytes())
        .and_then(move |body: warp::hyper::body::Bytes| {
            let handler = upload_handler.clone();
            async move { handler.put(body).await }
        });

    let fetch = warp::path!(String)
        .and(warp::get())
        .and_then(move |hex_key: String| {
            let handler = fetch_handler.clone();
            async move { handler.get(hex_key).await }
        });

    warp::serve(upload.or(fetch))
        .run(([127, 0, 0, 1], 3031))
        .await;

    Ok(())
}

#[derive(Debug)]
struct UploadError;
impl warp::reject::Reject for UploadError {}

struct UploadHandler {
    pool: Arc<sqlx::sqlite::SqlitePool>
}

impl UploadHandler {
    fn new(pool: Arc<sqlx::sqlite::SqlitePool>) -> Self {
        Self {
            pool: pool,
        }
    }

    async fn put(&self, body: warp::hyper::body::Bytes) -> Result<impl warp::Reply, warp::Rejection> {
        let digest = Sha256::digest(&body);
        info!("Received PUT with digest {:x}", &digest);

        let mut conn = self.pool.acquire().await.map_err(|_| warp::reject::custom(UploadError))?;
        conn.transaction(move |conn: &mut sqlx::Transaction<'_, sqlx::Sqlite>| Box::pin(async move {
            let conn1: &mut sqlx::Transaction<'_, sqlx::Sqlite> = conn;
            let result = sqlx::query("INSERT INTO metadata (key) VALUES (?)")
                .bind(digest.as_slice())
                .execute(conn1).await;
            match &result {
                Ok(done) => {
                    let metadata_rows = done.rows_affected();
                    info!("inserted ({}) row(s) into metadata", metadata_rows);
                    if metadata_rows == 1 {
                        let blob_rows = sqlx::query("INSERT INTO blobs (key, data) VALUES (?, ?)")
                            .bind(digest.as_slice())
                            .bind(body.as_ref())
                            .execute(conn).await?;
                        info!("inserted ({}) row(s) into blobs", blob_rows.rows_affected());
                    } else {
                        warn!("unexpected metadata row insert count: {}", metadata_rows);
                    }
                    Ok(())
                },
                Err(sqlx::Error::Database(e)) => {
                    if let Some(code) = e.code() {
                        // 1555 is unique constraint violated, can just return OK since we
                        // already have the blob.
                        // https://sqlite.org/rescode.html#constraint_primarykey
                        if code == "1555" {
                            info!("blob {:x} already inserted into metadata, skipping", &digest);
                        } else {
                            error!("Database error: {:?}", e);
                            result?;
                        }
                    }
                    Ok(())
                }
                Err(e) => {
                    error!("Database error: {:?}", e);
                    result?;
                    Ok(())
                }
            }
        })).await.map_err(|_: sqlx::Error| warp::reject::custom(UploadError))?;

        Ok(format!("{:x}", digest))
    }
}

#[derive(Debug)]
struct FetchError;
impl warp::reject::Reject for FetchError {}

struct FetchHandler {
    pool: Arc<sqlx::sqlite::SqlitePool>
}

impl FetchHandler {
    fn new(pool: Arc<sqlx::sqlite::SqlitePool>) -> Self {
        Self {
            pool: pool,
        }
    }

    async fn get(&self, hex_key: String) -> Result<impl warp::Reply, warp::Rejection> {
        info!("Received GET for key {}", &hex_key);
        let key = hex::decode(&hex_key.to_lowercase())
            .map_err(|_| warp::reject::custom(FetchError))?;

        let mut conn = self.pool.acquire().await.map_err(|_| warp::reject::custom(FetchError))?;
        let (result,): (Option<Vec<u8>>,) = sqlx::query_as("SELECT data FROM blobs WHERE key = ?")
            .bind(&key)
            .fetch_one(&mut conn)
            .await
            .map_err(|_| warp::reject::custom(FetchError))?;

        result.ok_or_else(|| {
            info!("Did not find value for key {}", &hex_key);
            warp::reject::not_found()
        }).and_then(|value| {
            info!("Found value for key {}", &hex_key);
            let resp = http::response::Builder::new()
                .status(200)
                .body(value)
                .map_err(|_| warp::reject::custom(FetchError))?;
            Ok(resp)
        })
    }
}

use dotenv;
use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use tide::{Request, Response, Server, StatusCode};

#[cfg(test)]
mod tests;

#[async_std::main]
async fn main() {
    let app = server().await;

    app.listen("http://127.0.0.1:8080").await.unwrap();
}

#[cfg(not(test))]
async fn make_db_pool() -> Pool<Postgres> {
    let db_url = std::env::var("DATABASE_URL").unwrap();

    let db_pool: Pool<Postgres> = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .unwrap();

    db_pool
}

#[cfg(test)]
async fn make_db_pool() -> Pool<Postgres> {
    let db_url = std::env::var("DATABASE_URL_TEST").unwrap();

    let db_pool: Pool<Postgres> = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .unwrap();

    db_pool
}

async fn server() -> Server<State> {
    dotenv::dotenv().ok();
    pretty_env_logger::init();

    let db_pool = make_db_pool().await;
    let state = State { db_pool };

    let mut app = tide::with_state(state);

    app.at("/").get(|_req: Request<State>| async move {
        //let db_pool = req.state().db_pool;
        let json = json!([1, 2, 3]);
        let res = Response::builder(StatusCode::Ok)
            .body(json)
            .content_type("application/json")
            .build();

        Ok(res)
    });

    app
}

#[derive(Debug, Clone)]
struct State {
    db_pool: Pool<Postgres>,
}

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error[transparent]]
    DbError(#[from] sqlx::Error),

    #[error[transparent]]
    IoError(#[from] std::io::Error),

    #[error[transparent]]
    VarError(#[from] std::env::VarError),
}

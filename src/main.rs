use dotenv;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use sqlx::{query, query_as, Pool, Postgres};
use tide::{Request, Response, Server, StatusCode};
use uuid::Uuid;

#[cfg(test)]
mod tests;

#[async_std::main]
async fn main() {    
    dotenv::dotenv().ok();
    pretty_env_logger::init();

    let db_pool = make_db_pool().await;
    let app = server(db_pool).await;

    app.listen("http://127.0.0.1:8080").await.unwrap();
}

#[cfg(not(test))]
pub async fn make_db_pool() -> Pool<Postgres> {
    let db_url = std::env::var("DATABASE_URL").unwrap();

    let db_pool: Pool<Postgres> = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .unwrap();

    db_pool
}

#[cfg(test)]
pub async fn make_db_pool() -> Pool<Postgres> {
    let db_url = std::env::var("DATABASE_URL_TEST").unwrap();

    let db_pool: Pool<Postgres> = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .unwrap();

    db_pool
}

async fn server(db_pool: Pool<Postgres>) -> Server<State> {
    dotenv::dotenv().ok();
    pretty_env_logger::init();

    let state = State { db_pool };

    let mut app = tide::with_state(state);

    app.at("/users")
        .get(|req: Request<State>| async move {
            let db_pool = &req.state().db_pool;

            let users = query_as!(User, "select id, username from users")
                .fetch_all(db_pool)
                .await?;

            let res = Response::builder(StatusCode::Ok)
                .body(json!(&users))
                .content_type("application/json")
                .build();

            Ok(res)
        })
        .post(|mut req: Request<State>| async move {
            let db_pool = req.state().db_pool.clone();
            let create_user = req.body_json::<CreateUser>().await?;

            query!(
                r#"
                    insert into users (id, username)
                    values ($1, $2)
                "#,
                Uuid::new_v4(),
                create_user.username,
            )
            .execute(&db_pool)
            .await?;

            Ok(Response::new(StatusCode::Created))
        });

    app
}

#[derive(Debug, Clone)]
struct State {
    db_pool: Pool<Postgres>,
}

#[derive(Debug, Serialize)]
struct User {
    id: Uuid,
    username: String,
}

#[derive(Debug, Deserialize)]
struct CreateUser {
    username: String,
    password: String,
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

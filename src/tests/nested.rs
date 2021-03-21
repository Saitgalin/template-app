#[allow(unused_imports)]
use crate::{make_db_pool, server};
use assert_json_diff::{assert_json_eq, assert_json_include};
use serde_json::Value;
use sqlx::prelude::Connection;
use sqlx::{PgConnection, Postgres};
use std::env;
use tide::prelude::*;
use tide_testing::surf::{Response, StatusCode};
use tide_testing::TideTestingExt;

#[async_std::test]
async fn creating_a_user() -> Result<(), tide::Error> {
    dotenv::dotenv().ok();
    let db_pool = make_db_pool().await;
    let server = server(db_pool).await;

    let mut res = server.get("http://127.0.0.1:8080/users").await?;
    assert_eq!(res.status(), StatusCode::Ok);
    let json: Value = res.body_json().await.unwrap();
    assert_json_eq!(json, json!([]));

    let res = server
        .post("http://127.0.0.1:8080/users")
        .content_type("application/json")
        .body(json!({"username": "aygiz", "password": "123"}))
        .await?;

    assert_eq!(res.status(), StatusCode::Created);

    let mut res = server.get("http://127.0.0.1:8080/users").await?;
    assert_eq!(res.status(), StatusCode::Ok);
    let json: Value = res.body_json().await.unwrap();
    assert_json_include!(actual: json, expected: json!([{"username": "aygiz"}]));

    Ok(())
}

pub fn db_url() -> String {
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};

    let rng = thread_rng();
    let suffix: String = rng.sample_iter(&Alphanumeric).take(16).collect();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL missing from environment.");
    format!("{}_{}", db_url, suffix)
}

fn parse_db_url(db_url: &str) -> (&str, &str) {
    let separator_pos = db_url.rfind("/").unwrap();
    let pg_conn = &db_url[..=separator_pos];
    let db_name = &db_url[separator_pos + 1..];
    (pg_conn, db_name)
}

async fn create_db(db_url: &str) {
    let (pg_conn, db_name) = parse_db_url(db_url);

    let mut conn = PgConnection::connect(pg_conn).await.unwrap();

    let sql = &format!(r#"CREATE DATABASE "{}""#, &db_name);
    let query = sqlx::query::<Postgres>(sql)
        .execute(&mut conn)
        .await
        .unwrap();
}

async fn drop_db(db_url: &str) {
    let (pg_conn, db_name) = parse_db_url(db_url);
    let mut conn = PgConnection::connect(pg_conn).await.unwrap();

    // Disconnect any existing connections to the DB
    let sql = format!(
        r#"SELECT pg_terminate_backend(pg_stat_activity.pid)
FROM pg_stat_activity
WHERE pg_stat_activity.datname = '{db}'
AND pid <> pg_backend_pid();"#,
        db = db_name
    );
    sqlx::query::<Postgres>(&sql)
        .execute(&mut conn)
        .await
        .unwrap();

    // Clean it up, bubye!
    let sql = format!(r#"DROP DATABASE "{db}";"#, db = db_name);
    sqlx::query::<Postgres>(&sql)
        .execute(&mut conn)
        .await
        .unwrap();
}

pub async fn run_migrations(db_url: &str) {
    let (pg_conn, db_name) = parse_db_url(db_url);
    let mut conn = PgConnection::connect(&format!("{}/{}", pg_conn, db_name))
        .await
        .unwrap();

    let rows = sqlx::query!(
        "SELECT table_name FROM information_schema.tables WHERE table_schema='public'"
    )
    .fetch_all(&mut conn)
    .await
    .unwrap();
    dbg!(rows);

    // Run the migrations
    let sql = async_std::fs::read_to_string("bin/setup.sql")
        .await
        .unwrap();
        
    sqlx::query::<Postgres>(&sql)
        .execute(&mut conn)
        .await
        .unwrap();
}

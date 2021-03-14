#[allow(unused_imports)]
use crate::server;
use serde_json::Value;
use tide::prelude::*;
use tide_testing::TideTestingExt;

#[async_std::test]
async fn nested() {
    let server = server().await;

    let response_body: Value = server
        .get("http://127.0.0.1:8080/")
        .content_type("application/json")
        .recv_json()
        .await
        .unwrap();

    assert_eq!(response_body, json!([1, 2, 3]));
}
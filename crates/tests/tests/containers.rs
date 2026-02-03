//! Container infrastructure tests.
//!
//! Verifies that the testcontainers infrastructure works correctly.

use berry_tests::{get_chroma_url, start_chroma};

/// Test that ChromaDB container starts and is reachable.
#[tokio::test]
async fn test_chroma_container_starts() {
    let container = start_chroma().await;
    let url = get_chroma_url(&container).await;

    // Verify ChromaDB is reachable via heartbeat endpoint
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/api/v2/heartbeat", url))
        .send()
        .await
        .expect("Failed to connect to ChromaDB");

    assert!(
        response.status().is_success(),
        "ChromaDB heartbeat failed: {:?}",
        response.status()
    );

    // Parse heartbeat response
    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body.get("nanosecond heartbeat").is_some());
}

/// Test that ChromaDB API v1 is accessible.
#[tokio::test]
async fn test_chroma_api_v1() {
    let container = start_chroma().await;
    let url = get_chroma_url(&container).await;

    let client = reqwest::Client::new();

    // Test version endpoint
    let response = client
        .get(format!("{}/api/v1/version", url))
        .send()
        .await
        .expect("Failed to get version");

    assert!(response.status().is_success());
}

/// Test that we can create and list collections.
#[tokio::test]
async fn test_chroma_collections() {
    let container = start_chroma().await;
    let url = get_chroma_url(&container).await;

    let client = reqwest::Client::new();

    // Create a test collection (without metadata - ChromaDB doesn't allow empty metadata)
    let create_body = serde_json::json!({
        "name": "test_collection"
    });

    let response = client
        .post(format!("{}/api/v1/collections", url))
        .json(&create_body)
        .send()
        .await
        .expect("Failed to create collection");

    assert!(
        response.status().is_success(),
        "Failed to create collection: {:?}",
        response.text().await
    );

    // List collections
    let response = client
        .get(format!("{}/api/v1/collections", url))
        .send()
        .await
        .expect("Failed to list collections");

    assert!(response.status().is_success());

    let collections: Vec<serde_json::Value> = response.json().await.unwrap();
    assert!(!collections.is_empty());
    assert!(collections.iter().any(|c| c["name"] == "test_collection"));
}

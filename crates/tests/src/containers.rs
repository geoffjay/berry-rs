//! Container definitions for integration tests.
//!
//! Uses testcontainers to spin up real service instances for testing.

use testcontainers::{
    core::{ContainerPort, WaitFor},
    runners::AsyncRunner,
    ContainerAsync, GenericImage, ImageExt,
};

/// Default ChromaDB port.
pub const CHROMA_PORT: u16 = 8000;

/// ChromaDB container type alias.
pub type ChromaContainer = ContainerAsync<GenericImage>;

/// Start a ChromaDB container for testing.
///
/// Returns the container instance. The container will be automatically
/// stopped and removed when dropped.
///
/// # Example
///
/// ```ignore
/// let container = start_chroma().await;
/// let url = get_chroma_url(&container).await;
/// // Use ChromaDB at `url`...
/// ```
pub async fn start_chroma() -> ChromaContainer {
    // Use the official ChromaDB image
    // Pin to a specific version for reproducible tests
    let image = GenericImage::new("chromadb/chroma", "0.5.23")
        .with_exposed_port(ContainerPort::Tcp(CHROMA_PORT))
        .with_wait_for(WaitFor::message_on_stdout("Application startup complete"))
        .with_env_var("IS_PERSISTENT", "FALSE")
        .with_env_var("ANONYMIZED_TELEMETRY", "FALSE");

    image
        .start()
        .await
        .expect("Failed to start ChromaDB container")
}

/// Get the ChromaDB URL from a running container.
pub async fn get_chroma_url(container: &ChromaContainer) -> String {
    let host = container.get_host().await.unwrap();
    let port = container.get_host_port_ipv4(CHROMA_PORT).await.unwrap();
    format!("http://{}:{}", host, port)
}

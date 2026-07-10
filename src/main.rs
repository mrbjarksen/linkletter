use axum::{Router, routing::post, http::StatusCode};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/documents", post(process_document));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn process_document(content: String) -> Result<String, StatusCode> {
    Ok(content)
}

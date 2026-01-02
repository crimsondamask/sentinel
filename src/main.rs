use anyhow::Result;
use axum::{Router, response::Html, routing::get};

#[tokio::main]
async fn main() -> Result<()> {
    let app = Router::new().route("/", get(handler));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!("Listening on: {}", listener.local_addr()?);

    axum::serve(listener, app).await?;
    Ok(())
}

async fn handler() -> Html<&'static str> {
    Html("<h1>hello there <h1>")
}

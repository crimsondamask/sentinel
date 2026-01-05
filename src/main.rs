use anyhow::Result;
use axum::routing::post;
use axum::{Json, Router, routing::get};
use log::info;
use sentinel::state::GlobalState;
use sentinel::{DeviceLink, Link, ModbusTcpConfig, Protocol, api::*};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    let modbus_tcp_config = ModbusTcpConfig::new("127.0.0.1".to_owned(), 5502);
    let protocol = Protocol::ModbusTcp(modbus_tcp_config);

    let mut links = Vec::new();
    let modbus_link = Link::Device(DeviceLink::new(
        "MB_LINK".to_owned(),
        "LK1".to_owned(),
        0,
        protocol,
        1000,
    ));

    links.push(modbus_link);

    let state = GlobalState::new(links);

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .or_else(|_| EnvFilter::try_new("sentinel=info,tower_http=info"))
                .unwrap(),
        )
        .compact()
        .pretty()
        .init();

    let app = Router::new()
        .route("/api/get_device_link_config", post(get_device_link_config))
        .route("/api/get_tag_config", post(get_tag_config))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!("Listening on: {}", listener.local_addr()?);

    axum::serve(listener, app).await?;
    Ok(())
}

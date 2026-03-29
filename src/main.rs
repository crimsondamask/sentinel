use anyhow::Result;
use axum::routing::post;
use axum::{Json, Router, routing::get};
use sentinel::state::GlobalState;
use sentinel::{DeviceLink, EvalLink, InputsLink, Link, ModbusTcpConfig, Protocol, Task, api::*};
use tokio::fs;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    let mut links = Vec::new();

    let config_string = fs::read_to_string("./CurrentConfig/current_config.json");
    if let Ok(config_string) = config_string.await {
        if let Ok(data) = serde_json::from_str(config_string.as_str()) {
            links = data;
        }
    }

    if links.is_empty() {
        let modbus_tcp_config = ModbusTcpConfig::new("127.0.0.1".to_owned(), 5502);
        let protocol = Protocol::ModbusTcp(modbus_tcp_config);

        let modbus_link = Link::Device(DeviceLink::new(
            "MB_LINK".to_owned(),
            "LK1".to_owned(),
            0,
            protocol.clone(),
            1000,
            500,
        ));

        links.push(modbus_link);

        /*
        *
        let modbus_link = Link::Device(DeviceLink::new(
            "MB_LINK2".to_owned(),
            "LK2".to_owned(),
            1,
            protocol,
            1000,
            1000,
        ));
        links.push(modbus_link);
        */

        let inputs_link = Link::Inputs(InputsLink::new(
            1,
            "IN".to_owned(),
            "INPUTS".to_owned(),
            1000,
        ));
        links.push(inputs_link);

        let evals_link = Link::Eval(EvalLink::new(2, "EVAL".to_string(), 1000));
        links.push(evals_link);
    }

    let state = GlobalState::new(links.clone());

    // Spawn a task for each link.
    for link in links.iter() {
        let state_for_link = state.clone();
        match link {
            Link::Device(link) => {
                let task = Task::new(sentinel::TaskType::DeviceLink, state_for_link, link.id);
                sentinel::task::spawn(task)?;
            }
            Link::Inputs(link) => {
                let task = Task::new(sentinel::TaskType::Inputs, state_for_link, link.id);
                sentinel::task::spawn(task)?;
            }
            Link::Eval(link) => {
                let task = Task::new(sentinel::TaskType::Eval, state_for_link, link.id);
                sentinel::task::spawn(task)?;
            }
            _ => {}
        };
    }
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
        .route("/api/get_links_config", get(get_links_config))
        .route("/api/get_device_link_config", post(get_device_link_config))
        .route("/api/get_tag_config", post(get_tag_config))
        .route("/api/reconfigure_device_link", post(reconfig_device_link))
        .route("/api/reconfigure_device_tag", post(reconfig_device_tag))
        .route("/api/write_tag", post(write_link_tag))
        .route("/api/reconfig_links", post(reconfig_links))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!("Listening on: {}", listener.local_addr()?);

    axum::serve(listener, app).await?;
    Ok(())
}

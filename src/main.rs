use anyhow::Result;
use api::service::QueueServiceImpl;
use clap::Parser;
use infrastructure::postgres::{PgJobRepository, PgRenderRepository};
use libcubr::event::event::EventTransport;
use libcubr::event::nats::NATSEventTransport;
use libcubr::rpc::nats::NATSRPC;
use libcubr::rpc::rpc::RPCTransport;
use sqlx::postgres::PgPoolOptions;
use tracing::{error, info};

mod api;
mod config;
mod domain;
mod infrastructure;

#[tokio::main]
async fn main() -> Result<()> {
    let config = config::Config::parse();

    config::configure_tracing();

    let pool = PgPoolOptions::new().connect(&config.database_url).await?;
    let nc = async_nats::connect(&config.nats_url).await?;

    let event = NATSEventTransport::new(nc.clone(), "queue".to_string());
    let render = PgRenderRepository::new(pool.clone());
    let job = PgJobRepository::new(pool);
    let rpc = NATSRPC::new(nc, "queue".to_string());

    let service = QueueServiceImpl::new(render, job, event.clone());

    tokio::select! {
        _ = event.listen(service.clone()) => {
            error!("Event listener exited");
        }
        _ = rpc.listen(service) => {
            error!("RPC listener exited");
        }
    }

    info!("Exiting");

    Ok(())
}

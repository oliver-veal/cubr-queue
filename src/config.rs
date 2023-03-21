use clap::Parser;
use tracing_subscriber::{fmt::format, prelude::__tracing_subscriber_field_MakeExt, EnvFilter};

#[derive(Debug, Parser)]
pub struct Config {
    #[clap(required = true, env)]
    pub database_url: String,
    #[clap(default_value = "", env)]
    pub env: String,
    #[clap(default_value = "nats://localhost:4222", env)]
    pub nats_url: String,
}

pub fn configure_tracing() {
    let formatter =
        format::debug_fn(|writer, field, value| write!(writer, "{}={:?}", field, value))
            .delimited(" ");

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .fmt_fields(formatter)
        .init();
}

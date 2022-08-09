use tracing::instrument::WithSubscriber;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::prelude::*;
use crate::{BuildInfo, Config, error};

pub struct Metrics {
    pub registered_webhooks: opentelemetry::metrics::Counter<u64>,
    pub received_notifications: opentelemetry::metrics::Counter<u64>
}

pub struct State {
    pub config: Config,
    pub build_info: BuildInfo,
    redis: redis::Client,
    pub metrics: Option<Metrics>
}

build_info::build_info!(fn build_info);

pub fn new_state(config: Config, redis_client: redis::Client) -> State {
    let build_info: &BuildInfo = build_info();

    State {
        config,
        build_info: build_info.clone(),
        redis: redis_client,
        metrics: None
    }
}

impl State {
    pub fn get_redis_connection(&self) -> error::Result<redis::Connection> {
        let conn = self.redis.get_connection()?;
        Ok(conn)
    }

    pub fn set_telemetry(&mut self, tracer: opentelemetry::sdk::trace::Tracer, metrics: Metrics) {
        let otel_tracing_layer = tracing_opentelemetry::layer().with_tracer(tracer);

        tracing_subscriber::registry()
            .with(otel_tracing_layer)
            .init();

        self.metrics = Some(metrics);
    }
}
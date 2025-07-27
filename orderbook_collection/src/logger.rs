use tracing::Subscriber;
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Layer, Registry};

pub fn init(_name: impl Into<String>, level: impl Into<String>) {
    dotenvy::dotenv().ok();

    let subscriber = get_subscriber(level);
    LogTracer::init().expect("Failed to set logger");
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
}

fn get_subscriber(level: impl Into<String>) -> impl Subscriber {
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(level.into()))
        .unwrap_or_else(|_| EnvFilter::new("info"));
    Registry::default().with(filter).with(
        tracing_subscriber::fmt::layer()
            .with_thread_names(true)
            .with_thread_ids(true)
            .with_line_number(true)
            .with_file(true)
            .json()
            .flatten_event(true)
            .with_span_list(false)
            .boxed(),
    )
}

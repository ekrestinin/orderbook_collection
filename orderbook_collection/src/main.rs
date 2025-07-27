use orderbook_collection_lib::logger;
use serde::Deserialize;
use std::path::PathBuf;
use structopt::StructOpt;
use tracing::info;

#[ctor::ctor]
fn init_logger() {
    logger::init("orderbook_collection", "info");
}

#[derive(Debug, StructOpt)]
#[structopt(name = "orderbook_collection", about = "orderbook collection usage.")]
struct Opt {
    // #[structopt(short = "s", long = "snapshot")]
    #[structopt(parse(from_os_str))]
    snapshot: Option<PathBuf>,
    #[structopt(parse(from_os_str))]
    incremental: Option<PathBuf>,
    #[structopt(short = "c", long = "config")]
    config: Option<String>,
    #[structopt(short = "a", long = "use_array")]
    use_array: bool,
}

pub fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();
    let snapshot_file = opt
        .snapshot
        .unwrap_or_else(|| PathBuf::from("orderbook_collection/resources/snapshot.bin"));
    let incremental_file = opt
        .incremental
        .unwrap_or_else(|| PathBuf::from("orderbook_collection/resources/incremental.bin"));
    let use_array = opt.use_array;
    let config_file = opt.config.unwrap_or_else(|| "orderbook_collection/config/test.yaml".into());

    let config: orderbook_collection_lib::config::Config =
        load_config(&config_file).unwrap_or_else(|_| {
            info!("Using default config");
            default_config()
        });
    info!("Config: {:?}", config);
    if use_array {
        info!("Using array orderbook");
        let order_books =
            orderbook_collection_lib::run_array(snapshot_file, incremental_file, config)?;
        info!("Order books: {:?}", order_books);
    } else {
        info!("Using btree orderbook");
        let order_books =
            orderbook_collection_lib::run_btree(snapshot_file, incremental_file, config)?;
        info!("Order books: {:?}", order_books);
    }
    Ok(())
}

pub fn load_config<T: for<'a> Deserialize<'a>>(source: &str) -> anyhow::Result<T> {
    Ok(config::Config::builder()
        .add_source(config::File::with_name(source))
        .build()?
        .try_deserialize::<T>()?)
}

pub fn default_config() -> orderbook_collection_lib::config::Config {
    orderbook_collection_lib::config::Config {
        instruments: std::collections::HashMap::new(),
        incremental_buffer_size: 2048,
    }
}

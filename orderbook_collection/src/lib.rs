use std::path::PathBuf;

use tracing::debug;

pub mod array_orderbook;
pub mod btree_orderbook;
pub mod config;
pub mod ser;
pub mod logger;

pub fn run_btree(
    snapshot_file: PathBuf,
    incremental_file: PathBuf,
    config: config::Config,
) -> Result<std::collections::HashMap<u64, btree_orderbook::orderbook::OrderBook>, anyhow::Error> {
    let mut order_books = btree_orderbook::ser::read_snapshot_file(snapshot_file)?;
    debug!("Read {} order books from snapshot file", order_books.len());
    btree_orderbook::ser::read_incremental_file(incremental_file, &mut order_books, config.incremental_buffer_size)?;
    debug!(
        "Processed incremental updates, total order books: {}",
        order_books.len()
    );
    Ok(order_books)
}

pub fn run_array(
    snapshot_file: PathBuf,
    incremental_file: PathBuf,
    config: config::Config,
) -> Result<std::collections::HashMap<u64, Box<array_orderbook::orderbook::OrderBook>>, anyhow::Error>
{
    let mut order_books =
        array_orderbook::ser::read_snapshot_file(snapshot_file, config.instruments)?;
    debug!("Read {} order books from snapshot file", order_books.len());
    array_orderbook::ser::read_incremental_file(incremental_file, &mut order_books, config.incremental_buffer_size)?;
    debug!(
        "Processed incremental updates, total order books: {}",
        order_books.len()
    );
    Ok(order_books)
}
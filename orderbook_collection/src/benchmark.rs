use core::f64;
use std::{collections::HashMap, io::Read};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use orderbook_collection_lib::{
    array_orderbook::{self},
    btree_orderbook::{
        self,
        ser::{
            incremental,
            snapshot::{self, SNAPSHOT_RECORD_SIZE},
        },
    },
};

pub fn read_u64_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("read_u64_benchmark");
    let buf = 100u64.to_le_bytes().to_vec();

    group.significance_level(0.01).sample_size(50000);
    group.bench_function("safe_read", |b| {
        b.iter(|| {
            _ = safe_read(black_box(&buf[0..]));
        })
    });
    group.bench_function("unsafe_read", |b| {
        b.iter(|| {
            _ = unsafe_read(black_box(&mut &buf[0..]));
        })
    });
    group.finish();
}

fn safe_read(buf: &[u8]) -> u64 {
    let value = btree_orderbook::ser::common::read_u64(&mut &buf[0..]).unwrap();
    value
}

fn unsafe_read(buf: &mut &[u8]) -> u64 {
    let ptr = buf.as_ptr();
    let value = array_orderbook::ser::common::read_u64(ptr, 0);
    value
}

pub fn load_benchmark(c: &mut Criterion) {
    let mut snapshot_buf = read_file("resources/snapshot.bin").unwrap();
    let mut incremental_buf = read_file("resources/incremental.bin").unwrap();

    let mut order_books = init_array_orderbooks();

    let mut group = c.benchmark_group("load_benchmark");
    group.significance_level(0.01).sample_size(50000);
    group.bench_function("btree_load", |b| {
        b.iter(|| {
            _ = btree_load_and_clear(
                black_box(&mut snapshot_buf),
                black_box(&mut incremental_buf),
            );
        })
    });
    group.bench_function("array_load", |b| {
        b.iter(|| {
            _ = array_load_and_clear(
                black_box(&mut order_books),
                black_box(&mut snapshot_buf),
                black_box(&mut incremental_buf),
            );
        })
    });
    group.finish();
}

pub fn read_levels_benchmark(c: &mut Criterion) {
    let mut snapshot_buf = read_file("resources/snapshot.bin").unwrap();

    let mut group = c.benchmark_group("read_levels_benchmark");

    let mut order_books = btree_load_snapshot(&mut snapshot_buf).unwrap();
    group.significance_level(0.01).sample_size(50000);
    group.bench_function("btree_read_levels", |b| {
        b.iter(|| {
            _ = btree_read_levels(black_box(&mut order_books));
        })
    });
    let mut order_books = init_array_orderbooks();
    array_load_snapshot(&mut order_books, &mut snapshot_buf).unwrap();
    group.bench_function("array_read_levels", |b| {
        b.iter(|| {
            _ = array_read_levels(black_box(&mut order_books));
        })
    });
    group.finish();
}

fn btree_read_levels(
    order_books: &mut HashMap<u64, orderbook_collection_lib::btree_orderbook::orderbook::OrderBook>,
) -> (f64, f64, u64, u64, f64, f64, u64, u64) {
    let mut best_bid_price = f64::MIN;
    let mut worst_bid_price = f64::MAX;
    let mut worst_ask_price = f64::MIN;
    let mut best_ask_price = f64::MAX;
    let mut max_bid_qty = 0;
    let mut min_bid_qty = u64::MAX;
    let mut max_ask_qty = 0;
    let mut min_ask_qty = u64::MAX;

    for order_book in order_books.values_mut() {
        let best_bid = order_book.best_bid();
        let worst_bid = order_book.worst_bid();
        let best_ask = order_book.best_ask();
        let worst_ask = order_book.worst_ask();
        if let Some((price, qty)) = best_bid {
            best_bid_price = price.max(best_bid_price);
            max_bid_qty = qty.max(max_bid_qty);
        }
        if let Some((price, qty)) = worst_bid {
            worst_bid_price = price.min(worst_bid_price);
            min_bid_qty = qty.min(min_bid_qty);
        }
        if let Some((price, qty)) = best_ask {
            best_ask_price = price.min(best_ask_price);
            max_ask_qty = qty.max(max_ask_qty);
        }
        if let Some((price, qty)) = worst_ask {
            worst_ask_price = price.max(worst_ask_price);
            min_ask_qty = qty.min(min_ask_qty);
        }
    }
    (
        best_bid_price,
        worst_bid_price,
        max_bid_qty,
        min_bid_qty,
        best_ask_price,
        worst_ask_price,
        max_ask_qty,
        min_ask_qty,
    )
}

fn array_read_levels(
    order_books: &mut HashMap<u64, Box<array_orderbook::orderbook::OrderBook>>,
) -> (f64, f64, u64, u64, f64, f64, u64, u64) {
    let mut best_bid_price = f64::MIN;
    let mut worst_bid_price = f64::MAX;
    let mut worst_ask_price = f64::MIN;
    let mut best_ask_price = f64::MAX;
    let mut max_bid_qty = 0;
    let mut min_bid_qty = u64::MAX;
    let mut max_ask_qty = 0;
    let mut min_ask_qty = u64::MAX;
    for order_book in order_books.values_mut() {
        let best_bid = order_book.best_bid();
        let worst_bid = order_book.worst_bid();
        let best_ask = order_book.best_ask();
        let worst_ask = order_book.worst_ask();
        if let Some((price, qty)) = best_bid {
            best_bid_price = price.max(best_bid_price);
            max_bid_qty = qty.max(max_bid_qty);
        }
        if let Some((price, qty)) = worst_bid {
            worst_bid_price = price.min(worst_bid_price);
            min_bid_qty = qty.min(min_bid_qty);
        }
        if let Some((price, qty)) = best_ask {
            best_ask_price = price.min(best_ask_price);
            max_ask_qty = qty.max(max_ask_qty);
        }
        if let Some((price, qty)) = worst_ask {
            worst_ask_price = price.max(worst_ask_price);
            min_ask_qty = qty.min(min_ask_qty);
        }
    }
    (
        best_bid_price,
        worst_bid_price,
        max_bid_qty,
        min_bid_qty,
        best_ask_price,
        worst_ask_price,
        max_ask_qty,
        min_ask_qty,
    )
}

fn read_file(filename: &str) -> anyhow::Result<Vec<u8>> {
    let file = std::fs::File::open(filename)?;
    let mut reader = std::io::BufReader::new(file);
    let mut snapshot_buf = Vec::new();
    reader.read_to_end(&mut snapshot_buf)?;
    Ok(snapshot_buf)
}

fn init_array_orderbooks() -> HashMap<u64, Box<array_orderbook::orderbook::OrderBook>> {
    let config_1 = orderbook_collection_lib::config::OrderBookConfig {
        id: 1,
        min_price: 4000.0,
        max_price: 7000.0,
        tick_size: 0.01,
    };
    let config_2 = orderbook_collection_lib::config::OrderBookConfig {
        id: 2,
        min_price: 599000.0,
        max_price: 602000.0,
        tick_size: 0.01,
    };

    let mut order_books: HashMap<u64, Box<array_orderbook::orderbook::OrderBook>> = HashMap::new();
    order_books.insert(
        1,
        Box::new(array_orderbook::orderbook::OrderBook::new(config_1)),
    );
    order_books.get_mut(&1).unwrap().init();
    order_books.insert(
        2,
        Box::new(array_orderbook::orderbook::OrderBook::new(config_2)),
    );
    order_books.get_mut(&2).unwrap().init();
    order_books
}

fn btree_load_and_clear(
    snapshot_buf: &mut [u8],
    incremental_buf: &mut [u8],
) -> Result<(), anyhow::Error> {
    let order_books = btree_load(snapshot_buf, incremental_buf)?;
    btree_clear(order_books);
    Ok(())
}

fn btree_load(
    snapshot_buf: &mut [u8],
    incremental_buf: &mut [u8],
) -> Result<
    HashMap<u64, orderbook_collection_lib::btree_orderbook::orderbook::OrderBook>,
    anyhow::Error,
> {
    let mut order_books = btree_load_snapshot(snapshot_buf)?;
    btree_update_incremental(incremental_buf, &mut order_books)?;
    Ok(order_books)
}

fn btree_update_incremental(
    incremental_buf: &mut [u8],
    order_books: &mut HashMap<u64, orderbook_collection_lib::btree_orderbook::orderbook::OrderBook>,
) -> Result<(), anyhow::Error> {
    let mut offset = 0;
    Ok(while offset < incremental_buf.len() {
        offset += incremental::read(&incremental_buf[offset..], order_books)?;
    })
}

fn btree_load_snapshot(
    snapshot_buf: &mut [u8],
) -> Result<
    HashMap<u64, orderbook_collection_lib::btree_orderbook::orderbook::OrderBook>,
    anyhow::Error,
> {
    let mut order_books = HashMap::new();
    let mut offset = 0;
    while offset < snapshot_buf.len() {
        let orderbook = snapshot::read(&mut snapshot_buf[offset..offset + SNAPSHOT_RECORD_SIZE])?;
        offset += SNAPSHOT_RECORD_SIZE;
        order_books.insert(orderbook.id, orderbook);
    }
    Ok(order_books)
}

fn btree_clear(
    mut order_books: HashMap<u64, orderbook_collection_lib::btree_orderbook::orderbook::OrderBook>,
) {
    for orderbook in order_books.values_mut() {
        orderbook.clear();
    }
}

fn array_load_and_clear(
    order_books: &mut std::collections::HashMap<u64, Box<array_orderbook::orderbook::OrderBook>>,
    snapshot_buf: &mut [u8],
    incremental_buf: &mut [u8],
) -> Result<(), anyhow::Error> {
    array_load(order_books, snapshot_buf, incremental_buf)?;
    array_clear(order_books);
    Ok(())
}

fn array_load(
    order_books: &mut HashMap<u64, Box<array_orderbook::orderbook::OrderBook>>,
    snapshot_buf: &mut [u8],
    incremental_buf: &mut [u8],
) -> Result<(), anyhow::Error> {
    array_load_snapshot(order_books, snapshot_buf)?;
    array_update_incremental(order_books, incremental_buf)
}

fn array_update_incremental(
    order_books: &mut HashMap<u64, Box<array_orderbook::orderbook::OrderBook>>,
    incremental_buf: &mut [u8],
) -> Result<(), anyhow::Error> {
    let mut offset = 0;
    Ok(while offset < incremental_buf.len() {
        offset += array_orderbook::ser::incremental::read(&incremental_buf[offset..], order_books)?;
    })
}

fn array_load_snapshot(
    order_books: &mut HashMap<u64, Box<array_orderbook::orderbook::OrderBook>>,
    snapshot_buf: &mut [u8],
) -> Result<(), anyhow::Error> {
    let mut offset = 0;
    Ok(while offset < snapshot_buf.len() {
        array_orderbook::ser::snapshot::read(
            &mut snapshot_buf[offset..offset + SNAPSHOT_RECORD_SIZE],
            order_books,
        )?;
        offset += SNAPSHOT_RECORD_SIZE;
    })
}

fn array_clear(
    order_books: &mut std::collections::HashMap<u64, Box<array_orderbook::orderbook::OrderBook>>,
) {
    for orderbook in order_books.values_mut() {
        orderbook.clear();
    }
}

criterion_group!(
    benches,
    read_u64_benchmark,
    read_levels_benchmark,
    load_benchmark
);
criterion_main!(benches);

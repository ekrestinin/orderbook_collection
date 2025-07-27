use std::path::PathBuf;

use orderbook_collection_lib::{config, run_array, run_btree};

#[test]
fn test_run_btree() {
    let snapshot_file = PathBuf::from("resources/snapshot.bin");
    let incremental_file = PathBuf::from("resources/incremental.bin");
    let config = config::Config {
        instruments: std::collections::HashMap::new(),
        incremental_buffer_size: 256, //smaller buffer size to test reader reset
    };
    let order_books = run_btree(snapshot_file, incremental_file, config).unwrap();

    assert_eq!(order_books.len(), 2);
    assert_eq!(format!("{:?}", order_books.get(&1).unwrap()), "OrderBook(id: 1, seq_no: 51, timestamp: 1705717811000, bids: [(5000.75, 1300), (5000.7, 1300), (5000.65, 1200), (5000.6, 1100), (5000.55, 1000)], asks: [(5001.0, 2000), (5001.1, 2100), (5001.2, 2200), (5001.3, 2300), (5001.4, 2400)])");
    assert_eq!(format!("{:?}", order_books.get(&2).unwrap()), "OrderBook(id: 2, seq_no: 50, timestamp: 1705717810000, bids: [(600000.0, 250), (599900.0, 200), (599800.0, 150), (599700.0, 180), (599600.0, 220)], asks: [(600500.0, 300), (600600.0, 400), (600700.0, 350), (600800.0, 420), (600900.0, 500)])");
}

#[test]
fn test_run_array() {
    let snapshot_file = PathBuf::from("resources/snapshot.bin");
    let incremental_file = PathBuf::from("resources/incremental.bin");
    let mut instruments = std::collections::HashMap::new();
    instruments.insert(
        1,
        config::OrderBookConfig {
            id: 1,
            min_price: 4000.0,
            max_price: 7000.0,
            tick_size: 0.01,
        },
    );
    instruments.insert(
        2,
        config::OrderBookConfig {
            id: 2,
            min_price: 599000.0,
            // min_price: 600000.0,
            max_price: 602000.0,
            tick_size: 0.01,
        },
    );
    let config = config::Config {
        instruments,
        incremental_buffer_size: 256, //smaller buffer size to test reader reset
    };
    let order_books = run_array(snapshot_file, incremental_file, config).unwrap();

    assert_eq!(order_books.len(), 2);
    assert_eq!(format!("{:?}", order_books.get(&1).unwrap()), "OrderBook(id: 1, seq_no: 51, timestamp: 1705717811000, bids: [(5000.75, 1300), (5000.7, 1300), (5000.65, 1200), (5000.6, 1100), (5000.55, 1000)], asks: [(5001.0, 2000), (5001.1, 2100), (5001.2, 2200), (5001.3, 2300), (5001.4, 2400)])");
    assert_eq!(format!("{:?}", order_books.get(&2).unwrap()), "OrderBook(id: 2, seq_no: 50, timestamp: 1705717810000, bids: [(600000.0, 250), (599900.0, 200), (599800.0, 150), (599700.0, 180), (599600.0, 220)], asks: [(600500.0, 300), (600600.0, 400), (600700.0, 350), (600800.0, 420), (600900.0, 500)])");
}

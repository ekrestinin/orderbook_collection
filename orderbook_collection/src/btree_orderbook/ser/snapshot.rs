use std::mem;

use tracing::{debug, trace};

use crate::{
    btree_orderbook::orderbook::OrderBook,
    btree_orderbook::ser::common::{read_f64, read_u64},
};

pub const SNAPSHOT_RECORD_SIZE: usize = 24 + 5 * (16 + 16); // 24 bytes for metadata, 5 pairs of (price, volume)

///
/// Reads the snapshot data from the buffer into the order book.
/// The buffer is expected to contain the following structure:
/// - 8 bytes for timestamp (u64)
/// - 8 bytes for sequence number (u64)
/// - 8 bytes for ID (u64)
/// - 5 pairs of 8 bytes for price (f64) and 8 bytes
///   for volume (u64) for bids and asks as following:
///   - bid1 price
///   - bid1 volume
///   - ask1 price
///   - ask1 volume
///   ...
///   - bid5 price
///   - bid5 volume
///   - ask5 price
///   - ask5 volume
pub fn read(buf: &[u8]) -> anyhow::Result<OrderBook> {
    let mut orderbook = OrderBook::default();
    
    let mut offset = 0;

    // reading metadata
    orderbook.timestamp = read_u64(&mut &buf[offset..])?;
    offset += mem::size_of::<u64>();
    orderbook.seq_no = read_u64(&mut &buf[offset..])?;
    offset += mem::size_of::<u64>();
    orderbook.id = read_u64(&mut &buf[offset..])?;
    offset += mem::size_of::<u64>();
    debug!(
        "Reading snapshot for order book ID: {}, timestamp: {}, seq_no: {}",
        orderbook.id, orderbook.timestamp, orderbook.seq_no
    );
    // reading bids and asks
    for _ in 0..5 {
        let price = read_f64(&mut &buf[offset..])?;
        offset += mem::size_of::<f64>();
        let qty = read_u64(&mut &buf[offset..])?;
        offset += mem::size_of::<u64>();
        trace!("Add bid: price = {}, volume = {}", price, qty);
        orderbook.add_bid(price, qty);

        let price = read_f64(&mut &buf[offset..])?;
        offset += mem::size_of::<f64>();
        let qty = read_u64(&mut &buf[offset..])?;
        offset += mem::size_of::<u64>();
        trace!("Add ask: price = {}, volume = {}", price, qty);
        orderbook.add_ask(price, qty);
    }

    Ok(orderbook)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_snapshot() {
        let mut buf: Vec<u8> = vec![];
        buf.extend_from_slice(&1u64.to_le_bytes()); // timestamp
        buf.extend_from_slice(&2u64.to_le_bytes()); // seq_no
        buf.extend_from_slice(&3u64.to_le_bytes()); // id
                                                    // Bid1
        buf.extend_from_slice(&100f64.to_le_bytes()); // bid1 price
        buf.extend_from_slice(&10u64.to_le_bytes()); // bid1 volume
                                                     // Ask1
        buf.extend_from_slice(&101f64.to_le_bytes()); // ask1 price
        buf.extend_from_slice(&5u64.to_le_bytes()); // ask1 volume
                                                    // Bid2
        buf.extend_from_slice(&102f64.to_le_bytes()); // bid2 price
        buf.extend_from_slice(&20u64.to_le_bytes()); // bid2 volume
                                                     // Ask2
        buf.extend_from_slice(&103f64.to_le_bytes()); // ask2 price
        buf.extend_from_slice(&15u64.to_le_bytes()); // ask2 volume
                                                     // Bid3
        buf.extend_from_slice(&104f64.to_le_bytes()); // bid3 price
        buf.extend_from_slice(&30u64.to_le_bytes()); // bid3 volume
                                                     // Ask3
        buf.extend_from_slice(&105f64.to_le_bytes()); // ask3 price
        buf.extend_from_slice(&25u64.to_le_bytes()); // ask3 volume
                                                     // Bid4
        buf.extend_from_slice(&106f64.to_le_bytes()); // bid4 price
        buf.extend_from_slice(&40u64.to_le_bytes()); // bid4 volume
                                                     // Ask4
        buf.extend_from_slice(&107f64.to_le_bytes()); // ask4 price
        buf.extend_from_slice(&35u64.to_le_bytes()); // ask4 volume
                                                     // Bid5
        buf.extend_from_slice(&108f64.to_le_bytes()); // bid5 price
        buf.extend_from_slice(&50u64.to_le_bytes()); // bid5 volume
                                                     // Ask5
        buf.extend_from_slice(&109f64.to_le_bytes()); // ask5 price
        buf.extend_from_slice(&45u64.to_le_bytes()); // ask5 volume

        let orderbook = read(&buf).unwrap();
        assert_eq!(orderbook.id, 3);
        assert_eq!(orderbook.seq_no, 2);
        assert_eq!(orderbook.timestamp, 1);
        assert_eq!(orderbook.get_bids().len(), 5);
        assert_eq!(orderbook.get_asks().len(), 5);
        assert_eq!(orderbook.get_bids()[0].0, 108.0);
        assert_eq!(orderbook.get_bids()[0].1, 50);
        assert_eq!(orderbook.get_bids()[1].0, 106.0);
        assert_eq!(orderbook.get_bids()[1].1, 40);
        assert_eq!(orderbook.get_bids()[2].0, 104.0);
        assert_eq!(orderbook.get_bids()[2].1, 30);
        assert_eq!(orderbook.get_bids()[3].0, 102.0);
        assert_eq!(orderbook.get_bids()[3].1, 20);
        assert_eq!(orderbook.get_bids()[4].0, 100.0);
        assert_eq!(orderbook.get_bids()[4].1, 10);

        assert_eq!(orderbook.get_asks()[0].0, 101.0);
        assert_eq!(orderbook.get_asks()[0].1, 5);
        assert_eq!(orderbook.get_asks()[1].0, 103.0);
        assert_eq!(orderbook.get_asks()[1].1, 15);
        assert_eq!(orderbook.get_asks()[2].0, 105.0);
        assert_eq!(orderbook.get_asks()[2].1, 25);
        assert_eq!(orderbook.get_asks()[3].0, 107.0);
        assert_eq!(orderbook.get_asks()[3].1, 35);
        assert_eq!(orderbook.get_asks()[4].0, 109.0);
        assert_eq!(orderbook.get_asks()[4].1, 45);
    }
}

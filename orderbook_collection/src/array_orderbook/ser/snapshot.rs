use crate::array_orderbook::{
    orderbook::OrderBook,
    ser::{
        common::{read_f64, read_u64},
        Error,
    },
};

///
/// Reads the snapshot data from the buffer into the order book.
/// The buffer is expected to contain the following structure:
/// - 8 bytes for timestamp (u64)
/// - 8 bytes for sequence number (u64)
/// - 8 bytes for ID (u64)
/// - 5 pairs of 8 bytes for price (f64) and 8 bytes
///   for qty (u64) for bids and asks as following:
///   - bid1 price
///   - bid1 qty
///   - ask1 price
///   - ask1 qty
///   ...
///   - bid5 price
///   - bid5 qty
///   - ask5 price
///   - ask5 qty
pub fn read(
    buf: &[u8],
    orderbooks: &mut std::collections::HashMap<u64, Box<OrderBook>>,
) -> anyhow::Result<(), Error> {
    let ptr = buf.as_ptr();
    // Read metadata
    let timestamp = read_u64(ptr, crate::ser::SNAPSHOT_TIMESTAMP_OFFSET);
    let seq_no = read_u64(ptr, crate::ser::SNAPSHOT_SEQ_NO_OFFSET);
    let id = read_u64(ptr, crate::ser::SNAPSHOT_ID_OFFSET);

    let orderbook = orderbooks
        .get_mut(&id)
        .ok_or_else(|| Error::OrderBookNotFound(id))?;
    orderbook.clear();
    orderbook.timestamp = timestamp;
    orderbook.seq_no = seq_no;
    // Read bids and asks
    let mut offset = crate::ser::SNAPSHOT_METADATA_SIZE;
    for _ in 0..5 {
        let price = read_f64(ptr, offset);
        offset += crate::ser::LEVEL_PRICE_SIZE;
        let qty = read_u64(ptr, offset);
        offset += crate::ser::LEVEL_QTY_SIZE;
        orderbook.add_bid(price, qty).map_err(|e| {
            Error::InvalidData(format!(
                "Failed to add bid: {}, price: {}, qty: {}",
                e, price, qty
            ))
        })?;
        let price = read_f64(ptr, offset);
        offset += crate::ser::LEVEL_PRICE_SIZE;
        let qty = read_u64(ptr, offset);
        offset += crate::ser::LEVEL_QTY_SIZE;
        orderbook.add_ask(price, qty).map_err(|e| {
            Error::InvalidData(format!(
                "Failed to add ask: {}, price: {}, qty: {}",
                e, price, qty
            ))
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    fn write_snapshot() -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::new();
        buf.extend_from_slice(&1u64.to_le_bytes()); // timestamp
        buf.extend_from_slice(&2u64.to_le_bytes()); // seq_no
        buf.extend_from_slice(&1u64.to_le_bytes()); // id
                                                    // Bid1
        buf.extend_from_slice(&100f64.to_le_bytes()); // bid1 price
        buf.extend_from_slice(&10u64.to_le_bytes()); // bid1 qty
                                                     // Ask1
        buf.extend_from_slice(&101f64.to_le_bytes()); // ask1 price
        buf.extend_from_slice(&5u64.to_le_bytes()); // ask1 qty
                                                    // Bid2
        buf.extend_from_slice(&102f64.to_le_bytes()); // bid2 price
        buf.extend_from_slice(&20u64.to_le_bytes()); // bid2 qty
                                                     // Ask2
        buf.extend_from_slice(&103f64.to_le_bytes()); // ask2 price
        buf.extend_from_slice(&15u64.to_le_bytes()); // ask2 qty
                                                     // Bid3
        buf.extend_from_slice(&104f64.to_le_bytes()); // bid3 price
        buf.extend_from_slice(&30u64.to_le_bytes()); // bid3 qty
                                                     // Ask3
        buf.extend_from_slice(&105f64.to_le_bytes()); // ask3 price
        buf.extend_from_slice(&25u64.to_le_bytes()); // ask3 qty
                                                     // Bid4
        buf.extend_from_slice(&106f64.to_le_bytes()); // bid4 price
        buf.extend_from_slice(&40u64.to_le_bytes()); // bid4 qty
                                                     // Ask4
        buf.extend_from_slice(&107f64.to_le_bytes()); // ask4 price
        buf.extend_from_slice(&35u64.to_le_bytes()); // ask4 qty
                                                     // Bid5
        buf.extend_from_slice(&108f64.to_le_bytes()); // bid5 price
        buf.extend_from_slice(&50u64.to_le_bytes()); // bid5 qty
                                                     // Ask5
        buf.extend_from_slice(&109f64.to_le_bytes()); // ask5 price
        buf.extend_from_slice(&45u64.to_le_bytes()); // ask5 qty

        buf
    }

    fn init_orderbooks() -> std::collections::HashMap<u64, Box<OrderBook>> {
        let mut orderbooks = std::collections::HashMap::new();
        let config = crate::config::OrderBookConfig {
            id: 1,
            min_price: 90.0,
            max_price: 110.0,
            tick_size: 0.01,
        };
        orderbooks.insert(1, Box::new(OrderBook::new(config)));
        orderbooks.get_mut(&1).unwrap().init();
        orderbooks
    }

    #[test]
    fn test_read_snapshot() {
        let buf = write_snapshot();
        let mut orderbooks = init_orderbooks();

        read(&buf, &mut orderbooks).unwrap();
        let orderbook = orderbooks.get(&1).unwrap();
        assert_eq!(orderbook.id(), 1);
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

    #[test]
    fn test_read_snapshot_price_out_of_bounds() {
        let mut orderbooks = init_orderbooks();

        let mut buf = write_snapshot();
        // Modify the bid price to be out of bounds
        let mut ptr = buf.as_mut_ptr();
        let offset = crate::ser::SNAPSHOT_METADATA_SIZE; // Bid1
        let price_out_of_bounds = 200f64; // Out of bounds price
        unsafe {
            ptr = ptr.add(offset);
            std::ptr::write(ptr as *mut f64, price_out_of_bounds);
        };

        let result = read(&buf, &mut orderbooks);
        assert!(matches!(result, Err(Error::InvalidData(_))));

        let mut buf = write_snapshot();
        // Modify the ask price to be out of bounds
        let mut ptr = buf.as_mut_ptr();
        let offset = crate::ser::SNAPSHOT_METADATA_SIZE
            + crate::ser::LEVEL_PRICE_SIZE
            + crate::ser::LEVEL_QTY_SIZE; // Ask1
        let price_out_of_bounds = 200f64; // Out of bounds price
        unsafe {
            ptr = ptr.add(offset);
            std::ptr::write(ptr as *mut f64, price_out_of_bounds);
        };

        let result = read(&buf, &mut orderbooks);
        assert!(matches!(result, Err(Error::InvalidData(_))));
    }
}

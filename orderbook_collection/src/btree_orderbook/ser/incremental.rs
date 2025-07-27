use std::collections::HashMap;

use crate::{
    btree_orderbook::{
        orderbook::OrderBook,
        ser::common::{read_f64, read_u64},
    },
    ser::Error,
};

/// Reads the incremental update data from the buffer into the order book.
/// The buffer is expected to contain the following structure:
/// - 8 bytes for timestamp (u64)
/// - 8 bytes for sequence number (u64)
/// - 8 bytes for ID (u64)
/// - 8 bytes for number of updates (u64)
/// - For each update:
///   - 1 byte for side (0 for bid, 1 for ask)
///   - 8 bytes for price (f64)
///   - 8 bytes for volume (u64)
///
/// Exceptions:
/// * If the order book with the given ID does not exist, an error Error::OrderBookNotFound is returned.
/// * If the sequence number is older than the current sequence number of the order book,
/// the update is skipped.
/// * If the sequence number is greater than the current sequence number + 1,
/// the update is also skipped.
/// * If the buffer is too small to contain the updates, an error Error::BufferTooSmall is returned.
/// * If the data is invalid (e.g., cannot read price or volume), an error Error::InvalidData is returned.
///
/// Otherwise, the updates are applied to the order book.
pub fn read(buf: &[u8], orderbooks: &mut HashMap<u64, OrderBook>) -> anyhow::Result<usize, Error> {
    if buf.len() < crate::ser::UPDATE_METADATA_SIZE + crate::ser::UPDATE_LEVEL_SIZE {
        return Err(Error::BufferTooSmall);
    }
    // reading metadata
    let timestamp = read_u64(&mut &buf[crate::ser::UPDATE_TIMESTAMP_OFFSET..])
        .map_err(|_| Error::InvalidData("Failed to read timestamp".into()))?;
    let seq_no = read_u64(&mut &buf[crate::ser::UPDATE_SEQ_NO_OFFSET..])
        .map_err(|_| Error::InvalidData("Failed to read sequence number".into()))?;
    let id = read_u64(&mut &buf[crate::ser::UPDATE_ID_OFFSET..])
        .map_err(|_| Error::InvalidData("Failed to read ID".into()))?;
    let num_updates = read_u64(&mut &buf[crate::ser::UPDATE_NUM_UPDATES_OFFSET..])
        .map_err(|_| Error::InvalidData("Failed to read number of updates".into()))?
        as usize;
    let mut offset = crate::ser::UPDATE_METADATA_SIZE;
    // check if the buffer is large enough for the updates
    if buf.len() < offset + num_updates * crate::ser::UPDATE_LEVEL_SIZE {
        return Err(Error::BufferTooSmall);
    }
    // get order book and check if update is valid
    let orderbook = orderbooks
        .get_mut(&id)
        .ok_or_else(|| Error::OrderBookNotFound(id))?;
    // update is stale - skip it
    if seq_no < orderbook.seq_no {
        return Ok(offset + num_updates * crate::ser::UPDATE_LEVEL_SIZE);
    }
    // there's a gap - skip the update
    if seq_no > orderbook.seq_no + 1 {
        return Err(Error::GapDetected(
            id,
            offset + num_updates * crate::ser::UPDATE_LEVEL_SIZE,
        ));
    }
    orderbook.timestamp = timestamp;
    orderbook.seq_no = seq_no;

    // reading updates
    for _ in 0..num_updates {
        let side = buf[offset];
        offset += crate::ser::LEVEL_SIDE_SIZE;
        let price = read_f64(&mut &buf[offset..])
            .map_err(|_| Error::InvalidData("Failed to read price".into()))?;
        offset += crate::ser::LEVEL_PRICE_SIZE;
        let volume = read_u64(&mut &buf[offset..])
            .map_err(|_| Error::InvalidData("Failed to read volume".into()))?;
        offset += crate::ser::LEVEL_QTY_SIZE;
        if side == 0 {
            orderbook.add_bid(price, volume);
        } else {
            orderbook.add_ask(price, volume);
        }
    }
    Ok(offset)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::btree_orderbook::orderbook::OrderBook;

    fn init_orderbooks() -> HashMap<u64, OrderBook> {
        let mut order_books = HashMap::new();
        let order_book = OrderBook::new(3);
        order_books.insert(3, order_book);

        order_books.get_mut(&3).unwrap().seq_no = 1; // Set initial seq_no
        order_books.get_mut(&3).unwrap().timestamp = 1; // Set initial timestamp

        order_books.get_mut(&3).unwrap().add_bid(100.0, 10); // Add initial bid
        order_books.get_mut(&3).unwrap().add_ask(101.0, 5); // Add initial ask

        order_books
    }

    fn write_update(id: u64, timestamp: u64, seq_no: u64, updates: &[(u8, f64, u64)]) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&timestamp.to_le_bytes());
        buf.extend_from_slice(&seq_no.to_le_bytes());
        buf.extend_from_slice(&id.to_le_bytes());
        buf.extend_from_slice(&(updates.len() as u64).to_le_bytes());

        for (side, price, qty) in updates {
            buf.push(*side);
            buf.extend_from_slice(&price.to_le_bytes());
            buf.extend_from_slice(&qty.to_le_bytes());
        }
        buf
    }

    #[test]
    fn test_read_incremental() {
        let mut order_books = init_orderbooks();

        let buf = write_update(3, 2, 2, &[(0, 100f64, 10), (1, 101f64, 5)]);

        let offset = read(&buf, &mut order_books).unwrap();

        assert_eq!(offset, buf.len());
        assert_eq!(order_books.len(), 1);
        let order_book = order_books.get(&3).unwrap();
        assert_eq!(order_book.id, 3);
        assert_eq!(order_book.seq_no, 2);
        assert_eq!(order_book.timestamp, 2);
        assert_eq!(order_book.get_bids().len(), 1);
        assert_eq!(order_book.get_bids()[0].0, 100.0);
        assert_eq!(order_book.get_bids()[0].1, 10);
        assert_eq!(order_book.get_asks().len(), 1);
        assert_eq!(order_book.get_asks()[0].0, 101.0);
        assert_eq!(order_book.get_asks()[0].1, 5);
    }

    #[test]
    fn test_read_incremental_with_skipped_seq_no() {
        let mut order_books = init_orderbooks();

        let buf = write_update(
            3,
            2,
            4, // seq_no is greater than current seq_no + 1
            &[(0, 100f64, 15)],
        );

        let result = read(&buf, &mut order_books);

        match result {
            Err(Error::GapDetected(_, off)) if off == buf.len() => {}
            _ => panic!("Expected GapDetected error with correct offset"),
        }

        assert_eq!(order_books.len(), 1);
        let order_book = order_books.get(&3).unwrap();
        assert_eq!(order_book.id, 3);
        assert_eq!(order_book.seq_no, 1);
        assert_eq!(order_book.timestamp, 1);
        assert_eq!(order_book.get_bids().len(), 1);
        assert_eq!(order_book.get_bids()[0].0, 100.0);
        assert_eq!(order_book.get_bids()[0].1, 10);
        assert_eq!(order_book.get_asks().len(), 1);
        assert_eq!(order_book.get_asks()[0].0, 101.0);
        assert_eq!(order_book.get_asks()[0].1, 5);
    }

    #[test]
    fn test_read_incremental_with_older_seq_no() {
        let mut order_books = init_orderbooks();

        order_books.get_mut(&3).unwrap().seq_no = 3; // Set initial seq_no
        order_books.get_mut(&3).unwrap().timestamp = 2; // Set initial timestamp

        let buf = write_update(
            3,
            1, // timestamp
            2, // seq_no is older than current seq_no
            &[(0, 100f64, 15)],
        );

        let offset = read(&buf, &mut order_books).unwrap();
        assert_eq!(offset, buf.len());
        assert_eq!(order_books.len(), 1);
        let order_book = order_books.get(&3).unwrap();
        assert_eq!(order_book.id, 3);
        assert_eq!(order_book.seq_no, 3);
        assert_eq!(order_book.timestamp, 2);
        assert_eq!(order_book.get_bids().len(), 1);
        assert_eq!(order_book.get_bids()[0].0, 100.0);
        assert_eq!(order_book.get_bids()[0].1, 10);
        assert_eq!(order_book.get_asks().len(), 1);
        assert_eq!(order_book.get_asks()[0].0, 101.0);
        assert_eq!(order_book.get_asks()[0].1, 5);
    }
}

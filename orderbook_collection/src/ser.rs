use std::mem;

pub const UPDATE_LEVEL_SIZE: usize =
    mem::size_of::<u8>() + mem::size_of::<f64>() + mem::size_of::<u64>(); // 1 byte for side + 8 bytes for price + 8 bytes for qty
pub const UPDATE_METADATA_SIZE: usize = mem::size_of::<u64>() * 4; // 8 bytes for timestamp + 8 bytes for seq_no + 8 bytes for ID + 8 bytes for number of updates

pub const UPDATE_TIMESTAMP_OFFSET: usize = 0;
pub const UPDATE_SEQ_NO_OFFSET: usize = UPDATE_TIMESTAMP_OFFSET + mem::size_of::<u64>();
pub const UPDATE_ID_OFFSET: usize = UPDATE_SEQ_NO_OFFSET + mem::size_of::<u64>();
pub const UPDATE_NUM_UPDATES_OFFSET: usize = UPDATE_ID_OFFSET + mem::size_of::<u64>();

pub const SNAPSHOT_METADATA_SIZE: usize =  mem::size_of::<u64>() * 3; // 8 bytes for timestamp + 8 bytes for seq_no + 8 bytes for ID
pub const SNAPSHOT_LEVELS_SIZE: usize = 10*(mem::size_of::<f64>() + mem::size_of::<u64>()); // 8 bytes for price + 8 bytes for qty, 5 per side (bid/ask)
pub const SNAPSHOT_RECORD_SIZE: usize = SNAPSHOT_METADATA_SIZE + SNAPSHOT_LEVELS_SIZE; // 24 bytes for metadata, 5 pairs of (price, qty)
pub const SNAPSHOT_TIMESTAMP_OFFSET: usize = 0;
pub const SNAPSHOT_SEQ_NO_OFFSET: usize = SNAPSHOT_TIMESTAMP_OFFSET + mem::size_of::<u64>();
pub const SNAPSHOT_ID_OFFSET: usize = SNAPSHOT_SEQ_NO_OFFSET + mem::size_of::<u64>();

pub const LEVEL_PRICE_SIZE: usize = mem::size_of::<f64>();
pub const LEVEL_QTY_SIZE: usize = mem::size_of::<u64>();
pub const LEVEL_SIDE_SIZE: usize = mem::size_of::<u8>();

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Order book with ID {0} not found")]
    OrderBookNotFound(u64),
    #[error("Buffer too small for incremental update")]
    BufferTooSmall,
    #[error("Invalid incremental update data: {0}")]
    InvalidData(String),
    #[error("Gap detected in incremental updates for order book ID {0}")]
    GapDetected(u64, usize),
}


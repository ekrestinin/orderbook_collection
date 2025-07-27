use std::{
    collections::HashMap,
    io::{Read, Seek, SeekFrom},
    path::PathBuf,
    vec,
};

use anyhow::bail;
use tracing::{info, trace, warn};

use crate::btree_orderbook::orderbook::OrderBook;

pub mod common;
pub mod incremental;
pub mod snapshot;

/// Reads the snapshot file and returns a map of order books indexed by their IDs.
/// The snapshot file is expected to contain serialized order book data in a specific format.
pub fn read_snapshot_file(snapshot_file: PathBuf) -> anyhow::Result<HashMap<u64, OrderBook>> {
    info!("Reading snapshot file: {:?}", snapshot_file);
    let mut order_books = HashMap::new();
    let file = std::fs::File::open(snapshot_file)?;
    let mut reader = std::io::BufReader::new(file);
    let mut buf: [u8; snapshot::SNAPSHOT_RECORD_SIZE] = [0; snapshot::SNAPSHOT_RECORD_SIZE];
    while reader.read_exact(&mut buf).is_ok() {
        let orderbook = snapshot::read(&buf)?;
        // Store the order book in the map using its ID
        order_books.insert(orderbook.id, orderbook);
    }
    Ok(order_books)
}

/// Reads the incremental updates from the file and applies them to the order books.
/// Exceptions:
/// * If the order book with the given ID does not exist, an error is returned.
/// * If invalid data is encountered, an error is returned.
/// The data is read in chunks, and each chunk is processed until the end of the file.
/// The buffer size is specified to optimize reading performance.
pub fn read_incremental_file(
    incremental_file: PathBuf,
    order_books: &mut HashMap<u64, OrderBook>,
    buffer_size: usize,
) -> anyhow::Result<()> {
    info!("Reading incremental file: {:?}", incremental_file);
    let file = std::fs::File::open(incremental_file)?;
    let mut reader = std::io::BufReader::new(file);
    let mut buf: Vec<u8> = vec![0; buffer_size];
    let mut reader_offset = 0;
    // Read the file in chunks
    while let Ok(bytes_read) = reader.read(&mut buf) {
        trace!("Read {} bytes from incremental file", bytes_read);
        // If no bytes were read, i.e. end of file, break the loop
        if bytes_read == 0 {
            break;
        }
        let mut offset = 0;
        while offset < bytes_read {
            match incremental::read(&buf[offset..bytes_read], order_books) {
                Ok(new_offset) => {
                    // If the read was successful, update the offset
                    offset += new_offset;
                    reader_offset += new_offset;
                    trace!(
                        "Processed {} bytes, total offset: {}",
                        new_offset,
                        reader_offset
                    );
                }
                Err(e) => {
                    match e {
                        crate::ser::Error::OrderBookNotFound(id) => {
                            // if found unexpected order book ID, bail out
                            bail!("Order book with ID {} not found", id);
                        }
                        crate::ser::Error::BufferTooSmall => {
                            // If the buffer is too small, need to seek back to the start of the current read and read the next chunk
                            trace!("Buffer too small for incremental update");
                            // reader.seek_relative(-(bytes_read as i64 - offset as i64))?;
                            reader.seek(SeekFrom::Current(-(bytes_read as i64 - offset as i64)))?;
                        }
                        crate::ser::Error::InvalidData(ref msg) => {
                            // If the data is invalid, log the error and bail out
                            bail!("Invalid incremental update data: {}", msg);
                        }
                        crate::ser::Error::GapDetected(id, new_offset) => {
                            // If a gap is detected in the incremental updates
                            // log a warning and read the next update
                            warn!(
                                "Gap detected in incremental updates for order book ID {}",
                                id
                            );
                            offset += new_offset;
                            reader_offset += new_offset;
                            continue;
                        }
                    }
                    break; // Exit the loop if an error occurs
                }
            }
        }
    }

    Ok(())
}

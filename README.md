# Overview

## Implementation details
There are two implementations:
* Using general purpose order book implementation based on BTreeMap. This implementation trades latency for smaller memory footprint and simpler code that is easier to maintain and reason about.
* Using more latency optimized order book implementation based on arrays. This implementation requires careful configuration and testing as it expects prices to be within pre-defined bounds and uses unsafe code. The trade off here is using more memory and more complicated code to achieve lower latency due to better CPU cache locality.

The implementations share very little code, as code reuse would require additional abstraction layers, which would impact performance, therefore there is some code duplication, particularly for reading snapshots and incremental updates.

## Performance
The performance difference between the two implementations was measured for:
* reading snapshot and applying incremental updates
* reading best and worst levels for both sides

The difference for both benchmarks is about 2x with order book based on arrays being faster.

Benchmark reports can be found in /benchmark

## Notes
* The output contains order books as of latest applied update with prices sorted by distance to mid.
* If there is a gap detected in incremental updates (orderbook seq_no + 1 < update seq_no), such updates and all following updates are dropped.

## Improvements
* Adaptive expansion of array based order book as prices shift. In case of price movement outside of pre-configured bounds, it is possible to rebuild array based order book with new bounds and copy snapshot from the old book.

# Usage
## Parameters
Usage:
```shell
cargo run --release --bin orderbook_collection -- <snapshot_file> \
<incremental file> \
[--use_array] \
[--config orderbook_collection/config/test.yaml] 
```
Example
```shell
cargo run --release --bin orderbook_collection -- orderbook_collection/resources/snapshot.bin \
orderbook_collection/resources/incremental.bin \
--use_array \
--config orderbook_collection/config/test.yaml
```
The parameters *use_arrays* and *config* are optional. If not specified, the BTreeMap implementation is used.

## Configuration
Configuration is optional and is only required for using array based order book implementation.
Configuration contains:
 - settings for array based order book, including:
    - list of instruments and for each instrument:    
        - price bounds (min/max price)
        - tick size
 - incremental_buffer_size specifies buffer size for reading incremental updates file.

Example:
```yaml
instruments:
  1:
    id: 1
    min_price: 4000.0
    max_price: 7000.0
    tick_size: 0.01
  2:
    id: 2
    min_price: 599000.0
    max_price: 602000.0
    tick_size: 0.01
incremental_buffer_size: 1024
```
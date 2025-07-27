use anyhow::bail;

use crate::config;

const MAX_LEVELS: usize = 1_000_000; // 1m levels, e.g. from 0 to 10_000 with 0.01 tick size
const EMPTY: usize = usize::MAX;


/// Array based order book implementation.
/// It uses a fixed size array to store order book levels, which allows for fast access and
/// updates and benefits from CPU cache locality. The order book is divided into bids and asks, each represented by a separate
/// `OrderBookSide`. The order book supports a configurable range of prices and tick size,
/// allowing for flexible market configurations.
pub struct OrderBook {
    pub seq_no: u64,
    pub timestamp: u64,
    pub bids: OrderBookSide,
    pub asks: OrderBookSide,
    config: config::OrderBookConfig,
}

impl OrderBook {
    pub fn new(config: config::OrderBookConfig) -> Self {
        let levels =
            ((config.max_price - config.min_price) / config.tick_size).round() as usize + 1;
        assert!(
            levels <= MAX_LEVELS,
            "Number of levels exceeds the max levels limit of {}",
            MAX_LEVELS
        );

        Self {
            bids: OrderBookSide::new(true),
            asks: OrderBookSide::new(false),
            config,
            seq_no: 0,
            timestamp: 0,
        }
    }

    pub fn init(&mut self) {
        let capacity =
            ((self.config.max_price - self.config.min_price) / self.config.tick_size) as usize;
        self.bids.init(capacity);
        self.asks.init(capacity);
    }

    pub fn id(&self) -> u64 {
        self.config.id
    }

    fn price_to_index(&self, price: f64) -> usize {
        if price < self.config.min_price || price > self.config.max_price {
            return EMPTY;
        }
        ((price - self.config.min_price) / self.config.tick_size).round() as usize
    }

    fn index_to_price(&self, index: usize) -> f64 {
        self.config.min_price + self.config.tick_size * index as f64
    }

    pub fn add_bid(&mut self, price: f64, qty: u64) -> anyhow::Result<()> {
        let idx = self.price_to_index(price);
        if idx == EMPTY {
            bail!("price is out of bounds");
        }
        self.bids.update(idx, qty);
        Ok(())
    }

    pub fn add_ask(&mut self, price: f64, qty: u64) -> anyhow::Result<()> {
        let idx = self.price_to_index(price);
        if idx == EMPTY {
            bail!("price is out of bounds");
        }
        self.asks.update(idx, qty);
        Ok(())
    }

    pub fn get_bids(&self) -> Vec<(f64, u64)> {
        self.bids
            .levels()
            .into_iter()
            .map(|(idx, qty)| (self.index_to_price(idx), qty))
            .collect()
    }
    pub fn get_asks(&self) -> Vec<(f64, u64)> {
        self.asks
            .levels()
            .into_iter()
            .map(|(idx, qty)| (self.index_to_price(idx), qty))
            .collect()
    }

    pub fn best_bid(&self) -> Option<(f64, u64)> {
        self.bids
            .head()
            .map(|(idx, qty)| (self.index_to_price(idx), qty))
    }

    pub fn best_ask(&self) -> Option<(f64, u64)> {
        self.asks
            .head()
            .map(|(idx, qty)| (self.index_to_price(idx), qty))
    }

    pub fn worst_bid(&self) -> Option<(f64, u64)> {
        self.bids
            .tail()
            .map(|(idx, qty)| (self.index_to_price(idx), qty))
    }

    pub fn worst_ask(&self) -> Option<(f64, u64)> {
        self.asks
            .tail()
            .map(|(idx, qty)| (self.index_to_price(idx), qty))
    }

    pub fn clear(&mut self) {
        self.bids.clear();
        self.asks.clear();
        self.seq_no = 0;
        self.timestamp = 0;
    }
}

impl std::fmt::Debug for OrderBook {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "OrderBook(id: {}, seq_no: {}, timestamp: {}, bids: {:?}, asks: {:?})",
            self.config.id,
            self.seq_no,
            self.timestamp,
            self.get_bids(),
            self.get_asks()
        )
    }
}

/// Represents a side of the order book (bids or asks).
/// It uses a linked list structure to maintain the order of levels, allowing for efficient insertion and
/// removal of levels. The linked list is based on array, which benefits from CPU cache locality in case of dense order book.
/// The `OrderBookSide` supports both ascending and descending order for bids and asks,
/// respectively, and provides methods to update levels, retrieve the head and tail of the side, and clear the side.

pub struct OrderBookSide {
    volumes: Vec<u64>,
    next: Vec<usize>,
    prev: Vec<usize>,
    head: usize,
    is_descending: bool,
}

impl OrderBookSide {
    pub fn new(is_descending: bool) -> Self {
        Self {
            volumes: vec![],
            next: vec![],
            prev: vec![],
            head: EMPTY,
            is_descending,
        }
    }

    pub fn init(&mut self, capacity: usize) {
        self.volumes = vec![0; capacity];
        self.next = vec![EMPTY; capacity];
        self.prev = vec![EMPTY; capacity];
        self.head = EMPTY;
    }

    fn insert(&mut self, index: usize) {
        if self.head == EMPTY {
            self.head = index;
            return;
        }

        let mut current = self.head;
        while current != EMPTY {
            if (self.is_descending && index > current) || (!self.is_descending && index < current) {
                // insert before i
                self.next[index] = current;
                if self.prev[current] != EMPTY {
                    self.next[self.prev[current]] = index;
                    self.prev[index] = self.prev[current];
                } else {
                    self.head = index;
                }
                self.prev[current] = index;
                return;
            }
            current = self.next[current];
        }

        // insert at the end
        current = self.head;
        while current != EMPTY {
            if self.next[current] == EMPTY {
                self.next[current] = index;
                self.prev[index] = current;
                break;
            }
            current = self.next[current];
        }
    }

    fn remove(&mut self, index: usize) {
        if self.prev[index] != EMPTY {
            self.next[self.prev[index]] = self.next[index];
        } else {
            self.head = self.next[index];
        }

        if self.next[index] != EMPTY {
            self.prev[self.next[index]] = self.prev[index];
        }

        self.next[index] = EMPTY;
        self.prev[index] = EMPTY;
    }

    pub fn update(&mut self, index: usize, qty: u64) {
        let prev_qty = self.volumes[index];
        self.volumes[index] = qty;

        if prev_qty == 0 && qty > 0 {
            self.insert(index);
        } else if prev_qty > 0 && qty == 0 {
            self.remove(index);
        }
    }

    pub fn levels(&self) -> Vec<(usize, u64)> {
        let mut levels = Vec::new();
        let mut current = self.head;
        while current != EMPTY {
            levels.push((current, self.volumes[current]));
            current = self.next[current];
        }
        levels
    }

    pub fn head(&self) -> Option<(usize, u64)> {
        if self.head != EMPTY {
            Some((self.head, self.volumes[self.head]))
        } else {
            None
        }
    }

    pub fn tail(&self) -> Option<(usize, u64)> {
        if self.head == EMPTY {
            None
        } else {
            let mut current = self.head;
            while self.next[current] != EMPTY {
                current = self.next[current];
            }
            Some((current, self.volumes[current]))
        }
    }

    pub fn clear(&mut self) {
        let mut current = self.head;
        while current != EMPTY {
            let next = self.next[current];
            self.remove(current);
            self.volumes[current] = 0;
            current = next;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::array_orderbook::orderbook::OrderBook;

    struct TestSet {
        order_book: Box<OrderBook>,
        initial_bids: Vec<(f64, u64)>,
        initial_asks: Vec<(f64, u64)>,
    }

    fn init_orderbook() -> TestSet {
        let initial_bids = vec![
            (100.1, 4),
            (100.05, 20),
            (100.0, 10),
        ];
        let initial_asks = vec![
            (101.0, 5),
            (101.1, 2),
            (102.0, 1),
        ];
        let config = crate::config::OrderBookConfig {
            id: 0,
            min_price: 90.0,
            max_price: 110.0,
            tick_size: 0.01,
        };
        let mut order_book = Box::new(OrderBook::new(config));
        order_book.init();

        for (bid_price, bid_qty) in &initial_bids {
            order_book.add_bid(*bid_price, *bid_qty).unwrap();
        }
        for (ask_price, ask_qty) in &initial_asks {
            order_book.add_ask(*ask_price, *ask_qty).unwrap();
        }

        TestSet {
            order_book,
            initial_bids,
            initial_asks,
        }
    }

    #[test]
    fn test_price_to_index() {
        let config = crate::config::OrderBookConfig {
            id: 0,
            min_price: 90.0,
            max_price: 110.0,
            tick_size: 0.01,
        };
        let order_book = OrderBook::new(config);
        assert_eq!(order_book.price_to_index(90.0), 0);
        assert_eq!(order_book.price_to_index(100.0), 1000);
        assert_eq!(order_book.price_to_index(110.0), 2000);
        assert_eq!(order_book.price_to_index(89.99), usize::MAX);
        assert_eq!(order_book.price_to_index(110.01), usize::MAX);
    }

    #[test]
    fn test_price_to_index_rounding() {
        let config = crate::config::OrderBookConfig {
            id: 0,
            min_price: 90.0,
            max_price: 110.0,
            tick_size: 0.01,
        };
        let order_book = OrderBook::new(config);
        let x = 0.1 + 0.2;
        let price = 90.0 + x;
        assert_eq!(order_book.price_to_index(price), 30);
        let price = 91.0 - x;
        assert_eq!(order_book.price_to_index(price), 70);
    }

    fn assert_order_book_levels(
        order_book: &OrderBook,
        expected_bids: &Vec<(f64, u64)>,
        expected_asks: &Vec<(f64, u64)>,
    ) {
        assert_eq!(order_book.get_bids(), expected_bids.clone());
        assert_eq!(order_book.get_asks(), expected_asks.clone());
        assert_eq!(order_book.best_bid(), expected_bids.first().cloned());
        assert_eq!(order_book.best_ask(), expected_asks.first().cloned());
        assert_eq!(order_book.worst_bid(), expected_bids.last().cloned());
        assert_eq!(order_book.worst_ask(), expected_asks.last().cloned());
    }

    #[test]
    fn test_order_book() {
        let test_set = init_orderbook();
        assert_order_book_levels(
            &test_set.order_book,
            &test_set.initial_bids,
            &test_set.initial_asks,
        );
    }

    #[test]
    fn test_order_book_update_existing_level() {
        let mut test_set = init_orderbook();

        let mut expected_bids = test_set.initial_bids.clone();

        expected_bids[2] = (expected_bids[2].0, expected_bids[2].1 + 5);
        // Update bid
        test_set.order_book.add_bid(expected_bids[2].0, expected_bids[2].1).unwrap();


        assert_order_book_levels(
            &test_set.order_book,
            &expected_bids,
            &test_set.initial_asks,
        );

        let mut expected_asks = test_set.initial_asks.clone();
        expected_asks[1] = (expected_asks[1].0, expected_asks[1].1 + 10);

        // Update ask
        test_set.order_book.add_ask(expected_asks[1].0, expected_asks[1].1).unwrap();
        assert_order_book_levels(
            &test_set.order_book,
            &expected_bids,
            &expected_asks,
        );
    }
    
    #[test]
    fn test_order_book_add_new_level() {
        let mut test_set = init_orderbook();
        // Insert new bid

        let mut expected_bids = test_set.initial_bids.clone();
        expected_bids.insert(1,(expected_bids[1].0 + 0.01, expected_bids[1].1 + 9));

        test_set.order_book.add_bid(expected_bids[1].0, expected_bids[1].1).unwrap();

        assert_order_book_levels(
            &test_set.order_book,
            &expected_bids,
            &test_set.initial_asks,
        );

        // Insert new ask

        let mut expected_asks = test_set.initial_asks.clone();
        expected_asks.insert(2,(expected_asks[2].0 - 0.01, expected_asks[2].1 + 3));

        test_set.order_book.add_ask(expected_asks[2].0, expected_asks[2].1).unwrap();

        assert_order_book_levels(
            &test_set.order_book,
            &expected_bids,
            &expected_asks,
        );
    }

    
    #[test]
    fn test_order_book_remove_levels() {
        let mut test_set = init_orderbook();

        // Remove bid
        let mut expected_bids = test_set.initial_bids.clone();
        test_set.order_book.add_bid(expected_bids[1].0, 0).unwrap(); // Remove bid
        expected_bids.remove(1);

        assert_order_book_levels(
            &test_set.order_book,
            &expected_bids,
            &test_set.initial_asks,
        );

        // Remove ask
        let mut expected_asks = test_set.initial_asks.clone();
        test_set.order_book.add_ask(expected_asks[1].0, 0).unwrap(); // Remove ask
        expected_asks.remove(1);

        assert_order_book_levels(
            &test_set.order_book,
            &expected_bids,
            &expected_asks,
        );
    }


    #[test]
    fn test_order_book_add_outside_price_range() {
        let mut test_set = init_orderbook();
        // add outside price range
        assert!(test_set.order_book.add_bid(89.0, 10).is_err());
        assert!(test_set.order_book.add_ask(111.0, 10).is_err());

        assert_order_book_levels(
            &test_set.order_book,
            &test_set.initial_bids,
            &test_set.initial_asks,
        );
    }

    #[test]
    fn test_order_book_add_best_levels() {
        let mut test_set = init_orderbook();

        // insert new best bid
        let mut expected_bids = test_set.initial_bids.clone();
        expected_bids.insert(0, (expected_bids[0].0 + 0.01, expected_bids[0].1 + 5));
        test_set.order_book.add_bid(expected_bids[0].0, expected_bids[0].1).unwrap();

        assert_order_book_levels(
            &test_set.order_book,
            &expected_bids,
            &test_set.initial_asks,
        );

        // insert new best ask
        let mut expected_asks = test_set.initial_asks.clone();
        expected_asks.insert(0, (expected_asks[0].0 - 0.01, expected_asks[0].1 + 3));
        test_set.order_book.add_ask(expected_asks[0].0, expected_asks[0].1).unwrap();

        assert_order_book_levels(
            &test_set.order_book,
            &expected_bids,
            &expected_asks,
        );

    }

    #[test]
    fn test_order_book_add_worst_levels() {
        let mut test_set = init_orderbook();

        //insert new worst bid
        let mut expected_bids = test_set.initial_bids.clone();
        expected_bids.push((expected_bids[2].0 - 0.01, expected_bids[2].1 + 1));
        test_set.order_book.add_bid(expected_bids[3].0, expected_bids[3].1).unwrap();
        assert_order_book_levels(
            &test_set.order_book,
            &expected_bids,
            &test_set.initial_asks,
        );

        //insert new worst ask
        let mut expected_asks = test_set.initial_asks.clone();
        expected_asks.push((expected_asks[2].0 + 0.01, expected_asks[2].1 + 2));
        test_set.order_book.add_ask(expected_asks[3].0, expected_asks[3].1).unwrap();
        assert_order_book_levels(
            &test_set.order_book,
            &expected_bids,
            &expected_asks,
        );
    }
    
    #[test]
    fn test_order_book_clear() {
        let mut test_set = init_orderbook();
        // clear
        test_set.order_book.clear();
        assert_eq!(test_set.order_book.get_bids().len(), 0);
        assert_eq!(test_set.order_book.get_asks().len(), 0);
        assert_eq!(test_set.order_book.best_bid(), None);
        assert_eq!(test_set.order_book.best_ask(), None);
        assert_eq!(test_set.order_book.worst_bid(), None);
        assert_eq!(test_set.order_book.worst_ask(), None);
    }
}

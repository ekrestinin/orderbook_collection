use std::collections::BTreeMap;

#[derive(Default)]
pub struct OrderBook {
    pub timestamp: u64,
    pub seq_no: u64,
    pub id: u64,
    pub bids: BTreeMap<PriceLevel, Level>,
    pub asks: BTreeMap<PriceLevel, Level>,
}

pub struct Level {
    pub price: f64,
    pub qty: u64,
}

impl Level {
    pub fn new(price: f64, volume: u64) -> Self {
        Self { price, qty: volume }
    }
}

#[derive(PartialEq)]
pub struct PriceLevel {
    pub price: f64,
}

impl PriceLevel {
    pub fn new(price: f64) -> Self {
        Self { price }
    }
}

impl Eq for PriceLevel {}

impl PartialOrd for PriceLevel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.price.partial_cmp(&other.price)
    }
}

impl Ord for PriceLevel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.price
            .partial_cmp(&other.price)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl OrderBook {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            seq_no: 0,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            timestamp: 0,
        }
    }

    pub fn add_bid(&mut self, price: f64, volume: u64) {
        if volume == 0u64 {
            self.bids.remove(&PriceLevel::new(price));
        } else {
            self.bids
                .insert(PriceLevel::new(price), Level::new(price, volume));
        }
    }

    pub fn add_ask(&mut self, price: f64, volume: u64) {
        if volume == 0u64 {
            self.asks.remove(&PriceLevel::new(price));
        } else {
            self.asks
                .insert(PriceLevel::new(price), Level::new(price, volume));
        }
    }

    pub fn clear(&mut self) {
        self.bids.clear();
        self.asks.clear();
    }

    pub fn get_bids(&self) -> Vec<(f64, u64)> {
        self.bids
            .values()
            .map(|x| (x.price, x.qty))
            .rev()
            .into_iter()
            .collect()
    }

    pub fn get_asks(&self) -> Vec<(f64, u64)> {
        self.asks
            .values()
            .map(|x| (x.price, x.qty))
            .into_iter()
            .collect()
    }

    pub fn best_bid(&self) -> Option<(f64, u64)> {
        self.bids
            .iter()
            .last()
            .map(|(_, level)| (level.price, level.qty))
    }

    pub fn best_ask(&self) -> Option<(f64, u64)> {
        self.asks
            .iter()
            .next()
            .map(|(_, level)| (level.price, level.qty))
    }

    pub fn worst_bid(&self) -> Option<(f64, u64)> {
        self.bids
            .iter()
            .next()
            .map(|(_, level)| (level.price, level.qty))
    }
    pub fn worst_ask(&self) -> Option<(f64, u64)> {
        self.asks
            .iter()
            .last()
            .map(|(_, level)| (level.price, level.qty))
    }
}

impl std::fmt::Debug for OrderBook {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "OrderBook(id: {}, seq_no: {}, timestamp: {}, bids: {:?}, asks: {:?})",
            self.id,
            self.seq_no,
            self.timestamp,
            self.get_bids(),
            self.get_asks()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestSet {
        order_book: OrderBook,
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
        let mut order_book = OrderBook::new(1);

        for (bid_price, bid_qty) in &initial_bids {
            order_book.add_bid(*bid_price, *bid_qty);
        }
        for (ask_price, ask_qty) in &initial_asks {
            order_book.add_ask(*ask_price, *ask_qty);
        }

        TestSet {
            order_book,
            initial_bids,
            initial_asks,
        }
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
        test_set.order_book.add_bid(expected_bids[2].0, expected_bids[2].1);


        assert_order_book_levels(
            &test_set.order_book,
            &expected_bids,
            &test_set.initial_asks,
        );

        let mut expected_asks = test_set.initial_asks.clone();
        expected_asks[1] = (expected_asks[1].0, expected_asks[1].1 + 10);

        // Update ask
        test_set.order_book.add_ask(expected_asks[1].0, expected_asks[1].1);
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

        test_set.order_book.add_bid(expected_bids[1].0, expected_bids[1].1);

        assert_order_book_levels(
            &test_set.order_book,
            &expected_bids,
            &test_set.initial_asks,
        );

        // Insert new ask

        let mut expected_asks = test_set.initial_asks.clone();
        expected_asks.insert(2,(expected_asks[2].0 - 0.01, expected_asks[2].1 + 3));

        test_set.order_book.add_ask(expected_asks[2].0, expected_asks[2].1);

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
        test_set.order_book.add_bid(expected_bids[1].0, 0); // Remove bid
        expected_bids.remove(1);

        assert_order_book_levels(
            &test_set.order_book,
            &expected_bids,
            &test_set.initial_asks,
        );

        // Remove ask
        let mut expected_asks = test_set.initial_asks.clone();
        test_set.order_book.add_ask(expected_asks[1].0, 0); // Remove ask
        expected_asks.remove(1);

        assert_order_book_levels(
            &test_set.order_book,
            &expected_bids,
            &expected_asks,
        );
    }

    #[test]
    fn test_order_book_add_best_levels() {
        let mut test_set = init_orderbook();

        // insert new best bid
        let mut expected_bids = test_set.initial_bids.clone();
        expected_bids.insert(0, (expected_bids[0].0 + 0.01, expected_bids[0].1 + 5));
        test_set.order_book.add_bid(expected_bids[0].0, expected_bids[0].1);

        assert_order_book_levels(
            &test_set.order_book,
            &expected_bids,
            &test_set.initial_asks,
        );

        // insert new best ask
        let mut expected_asks = test_set.initial_asks.clone();
        expected_asks.insert(0, (expected_asks[0].0 - 0.01, expected_asks[0].1 + 3));
        test_set.order_book.add_ask(expected_asks[0].0, expected_asks[0].1);

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
        test_set.order_book.add_bid(expected_bids[3].0, expected_bids[3].1);
        assert_order_book_levels(
            &test_set.order_book,
            &expected_bids,
            &test_set.initial_asks,
        );

        //insert new worst ask
        let mut expected_asks = test_set.initial_asks.clone();
        expected_asks.push((expected_asks[2].0 + 0.01, expected_asks[2].1 + 2));
        test_set.order_book.add_ask(expected_asks[3].0, expected_asks[3].1);
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

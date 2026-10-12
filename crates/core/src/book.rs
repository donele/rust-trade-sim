use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::hash::{Hash, Hasher};

use crate::symbols::SymbolRegistry;
use crate::{BookTicker, Incremental, Message, PacketBegin, Reset, Side, SymbolId, TimeNanos};

const INVALID_TICKS: i64 = i64::MAX;

/// A fixed-point-style value for deterministic book ordering.
///
/// `ticks * increment` is the display value. Book maps order values by ticks,
/// not by floating-point values.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Price {
    ticks: i64,
    increment: f64,
}

impl Price {
    pub fn from_ticks(ticks: i64, increment: f64) -> Self {
        Self { ticks, increment }
    }

    pub fn from_value(value: f64, increment: f64) -> Option<Self> {
        if !value.is_finite() || !increment.is_finite() || increment <= 0.0 {
            return None;
        }
        let ticks = (value / increment).round();
        if ticks < i64::MIN as f64 || ticks >= INVALID_TICKS as f64 {
            return None;
        }
        Some(Self::from_ticks(ticks as i64, increment))
    }

    pub fn ticks(self) -> i64 {
        self.ticks
    }

    pub fn increment(self) -> f64 {
        self.increment
    }

    pub fn to_value(self) -> f64 {
        self.ticks as f64 * self.increment
    }

    pub fn is_valid(self) -> bool {
        self.ticks != INVALID_TICKS
    }
}

impl Eq for Price {}

impl Ord for Price {
    fn cmp(&self, other: &Self) -> Ordering {
        self.ticks.cmp(&other.ticks)
    }
}

impl PartialOrd for Price {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for Price {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ticks.hash(state);
    }
}

pub type Quantity = Price;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BookLevel {
    pub price: Price,
    pub qty: Quantity,
    pub num_orders: u64,
    pub modified_time: TimeNanos,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BookError {
    InvalidPriceOrQuantity,
    UnknownSide,
    SymbolNotConfigured(SymbolId),
}

#[derive(Debug, Clone)]
pub struct OrderBook {
    pub symbol: SymbolId,
    price_increment: f64,
    qty_increment: f64,
    bids: BTreeMap<Price, BookLevel>,
    asks: BTreeMap<Price, BookLevel>,
    last_update_time: Option<TimeNanos>,
}

impl OrderBook {
    pub fn new(symbol: SymbolId, price_increment: f64, qty_increment: f64) -> Self {
        Self {
            symbol,
            price_increment,
            qty_increment,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            last_update_time: None,
        }
    }

    pub fn apply_incremental(&mut self, update: &Incremental) -> Result<(), BookError> {
        let price = Price::from_value(update.price, self.price_increment)
            .ok_or(BookError::InvalidPriceOrQuantity)?;
        let qty = Quantity::from_value(update.qty, self.qty_increment)
            .ok_or(BookError::InvalidPriceOrQuantity)?;
        let levels = match update.side {
            Side::Buy => &mut self.bids,
            Side::Sell => &mut self.asks,
            Side::Unknown => return Err(BookError::UnknownSide),
        };
        if qty.ticks() == 0 {
            levels.remove(&price);
        } else {
            let level = BookLevel {
                price,
                qty,
                num_orders: update.num_orders_at_level,
                modified_time: update.ts_nanos,
            };
            levels.insert(price, level);
        }
        self.last_update_time = Some(update.ts_nanos);
        Ok(())
    }

    pub fn apply_book_ticker(&mut self, ticker: &BookTicker) -> Result<(), BookError> {
        self.set_top(
            Side::Buy,
            ticker.best_bid_price,
            ticker.best_bid_qty,
            ticker.ts_nanos,
        )?;
        self.set_top(
            Side::Sell,
            ticker.best_ask_price,
            ticker.best_ask_qty,
            ticker.ts_nanos,
        )?;
        self.last_update_time = Some(ticker.ts_nanos);
        Ok(())
    }

    fn set_top(
        &mut self,
        side: Side,
        price_value: f64,
        qty_value: f64,
        ts_nanos: TimeNanos,
    ) -> Result<(), BookError> {
        let price = Price::from_value(price_value, self.price_increment)
            .ok_or(BookError::InvalidPriceOrQuantity)?;
        let qty = Quantity::from_value(qty_value, self.qty_increment)
            .ok_or(BookError::InvalidPriceOrQuantity)?;
        let levels = match side {
            Side::Buy => &mut self.bids,
            Side::Sell => &mut self.asks,
            Side::Unknown => return Err(BookError::UnknownSide),
        };

        match side {
            Side::Buy => levels.retain(|level_price, _| *level_price <= price),
            Side::Sell => levels.retain(|level_price, _| *level_price >= price),
            Side::Unknown => unreachable!(),
        }

        if qty.ticks() == 0 {
            levels.remove(&price);
        } else {
            levels.insert(
                price,
                BookLevel {
                    price,
                    qty,
                    num_orders: 0,
                    modified_time: ts_nanos,
                },
            );
        }
        Ok(())
    }

    pub fn reset(&mut self) {
        self.bids.clear();
        self.asks.clear();
        self.last_update_time = None;
    }

    pub fn best_bid(&self) -> Option<&BookLevel> {
        self.bids.values().next_back()
    }

    pub fn best_ask(&self) -> Option<&BookLevel> {
        self.asks.values().next()
    }

    pub fn bid(&self, price: Price) -> Option<&BookLevel> {
        self.bids.get(&price)
    }

    pub fn ask(&self, price: Price) -> Option<&BookLevel> {
        self.asks.get(&price)
    }

    pub fn bid_count(&self) -> usize {
        self.bids.len()
    }

    pub fn ask_count(&self) -> usize {
        self.asks.len()
    }

    pub fn last_update_time(&self) -> Option<TimeNanos> {
        self.last_update_time
    }
}

#[derive(Debug, Default, Clone)]
pub struct BookMap {
    books: HashMap<SymbolId, OrderBook>,
}

impl BookMap {
    pub fn from_registry(registry: &SymbolRegistry) -> Self {
        let mut books = Self::default();
        for symbol in registry.iter() {
            books.insert_symbol(symbol.id, symbol.price_increment, symbol.qty_increment);
        }
        books
    }

    pub fn insert_symbol(&mut self, symbol: SymbolId, price_increment: f64, qty_increment: f64) {
        self.books.insert(
            symbol,
            OrderBook::new(symbol, price_increment, qty_increment),
        );
    }

    pub fn get(&self, symbol: SymbolId) -> Option<&OrderBook> {
        self.books.get(&symbol)
    }

    pub fn get_mut(&mut self, symbol: SymbolId) -> Option<&mut OrderBook> {
        self.books.get_mut(&symbol)
    }

    pub fn process(&mut self, message: &Message) -> Result<(), BookError> {
        match message {
            Message::Incremental(update) => self.process_incremental(update),
            Message::BookTicker(update) => self.process_book_ticker(update),
            Message::PacketBegin(begin) if begin.snapshot => self.process_packet_begin(begin),
            Message::Reset(reset) => self.process_reset(reset),
            _ => Ok(()),
        }
    }

    fn process_incremental(&mut self, update: &Incremental) -> Result<(), BookError> {
        self.books
            .get_mut(&update.symbol)
            .ok_or(BookError::SymbolNotConfigured(update.symbol))?
            .apply_incremental(update)
    }

    fn process_book_ticker(&mut self, update: &BookTicker) -> Result<(), BookError> {
        self.books
            .get_mut(&update.symbol)
            .ok_or(BookError::SymbolNotConfigured(update.symbol))?
            .apply_book_ticker(update)
    }

    fn process_packet_begin(&mut self, begin: &PacketBegin) -> Result<(), BookError> {
        self.books
            .get_mut(&begin.symbol)
            .ok_or(BookError::SymbolNotConfigured(begin.symbol))?
            .reset();
        Ok(())
    }

    fn process_reset(&mut self, reset: &Reset) -> Result<(), BookError> {
        self.books
            .get_mut(&reset.symbol)
            .ok_or(BookError::SymbolNotConfigured(reset.symbol))?
            .reset();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn book() -> OrderBook {
        OrderBook::new(SymbolId(1), 0.1, 0.001)
    }

    #[test]
    fn prices_are_ordered_by_ticks() {
        let low = Price::from_value(100.01, 0.01).unwrap();
        let high = Price::from_value(100.02, 0.01).unwrap();
        assert!(low < high);
        assert_eq!(high.ticks(), 10002);
    }

    #[test]
    fn incremental_updates_and_removes_levels() {
        let mut book = book();
        let update = Incremental {
            ts_nanos: 10,
            symbol: SymbolId(1),
            price: 100.0,
            qty: 2.5,
            num_orders_at_level: 3,
            side: Side::Buy,
        };
        book.apply_incremental(&update).unwrap();
        assert_eq!(book.best_bid().unwrap().qty.to_value(), 2.5);

        let delete = Incremental { qty: 0.0, ..update };
        book.apply_incremental(&delete).unwrap();
        assert!(book.best_bid().is_none());
    }

    #[test]
    fn book_ticker_removes_stale_better_levels() {
        let mut book = book();
        for (price, side) in [(101.0, Side::Buy), (99.0, Side::Sell)] {
            book.apply_incremental(&Incremental {
                ts_nanos: 1,
                symbol: SymbolId(1),
                price,
                qty: 1.0,
                num_orders_at_level: 1,
                side,
            })
            .unwrap();
        }

        book.apply_book_ticker(&BookTicker {
            ts_nanos: 2,
            symbol: SymbolId(1),
            best_bid_price: 100.0,
            best_bid_qty: 2.0,
            best_ask_price: 100.5,
            best_ask_qty: 3.0,
            update_id: 1,
        })
        .unwrap();

        assert_eq!(book.best_bid().unwrap().price.to_value(), 100.0);
        assert_eq!(book.best_ask().unwrap().price.to_value(), 100.5);
    }

    #[test]
    fn packet_begin_snapshot_clears_book() {
        let mut books = BookMap::default();
        books.insert_symbol(SymbolId(1), 0.1, 0.001);
        books
            .process(&Message::Incremental(Incremental {
                ts_nanos: 1,
                symbol: SymbolId(1),
                price: 100.0,
                qty: 1.0,
                num_orders_at_level: 1,
                side: Side::Sell,
            }))
            .unwrap();
        books
            .process(&Message::PacketBegin(PacketBegin {
                symbol: SymbolId(1),
                ts_nanos: 2,
                snapshot: true,
            }))
            .unwrap();
        assert_eq!(books.get(SymbolId(1)).unwrap().ask_count(), 0);
    }
}

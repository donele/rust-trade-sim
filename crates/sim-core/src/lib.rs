pub type SimNanos = i64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SymbolId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OrderId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClientId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StrategyId(pub u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Trade {
    pub ts_nanos: SimNanos,
    pub symbol: SymbolId,
    pub price_ticks: i64,
    pub qty: i64,
    pub side: Side,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrderRequest {
    pub ts_nanos: SimNanos,
    pub client_id: ClientId,
    pub strategy_id: StrategyId,
    pub symbol: SymbolId,
    pub side: Side,
    pub price_ticks: i64,
    pub qty: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrderUpdate {
    pub ts_nanos: SimNanos,
    pub order_id: OrderId,
    pub filled_qty: i64,
    pub leaves_qty: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayEvent {
    Trade(Trade),
    OrderRequest(OrderRequest),
    OrderUpdate(OrderUpdate),
    EndTime { ts_nanos: SimNanos },
}


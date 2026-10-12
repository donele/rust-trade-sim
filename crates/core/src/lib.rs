//! Shared domain types used by live and simulated trading runtimes.

pub type TimeNanos = i64;
pub type TradeId = i64;

pub mod book;
pub mod messages;
pub mod symbols;

pub use messages::{
    AlgoOrderRequest, BookTicker, ClockMessage, EndTimeMessage, FundingRate, Incremental,
    Liquidation, Message, MessageType, OrderRequest, OrderUpdate, PacketBegin, PacketEnd, Reset,
    Trade,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SymbolId(pub u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OrderId(pub u64);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClientId(pub u64);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ParentOrderId(pub u64);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StrategyId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Side {
    Buy,
    Sell,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OrderStatus {
    None,
    New,
    Sent,
    Acked,
    PartialFill,
    Filled,
    CancelSent,
    CancelReject,
    Canceled,
    Closed,
    Reject,
    StatusReject,
    PendingCancel,
    PlacerAck,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RejectReason {
    None,
    RateLimitExceeded,
    InvalidPrice,
    InvalidQty,
    PostOnly,
    OrderNotFound,
    OrderNotAcked,
    OrderSizeTooLarge,
    RejectGeneric,
    PlacerRejectGeneric,
    NumberOfOrdersExceeded,
    NumberOfVenueOrdersExceeded,
    NoMarketDataUpdate,
    PositionLimitExceeded,
    PnlLimitExceeded,
    ControlNotAllowed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OrderType {
    None,
    Market,
    Limit,
    Internal,
    Manual,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeInForce {
    None,
    Ioc,
    Gtc,
    Fok,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OrderAction {
    None,
    New,
    Modify,
    Cancel,
    CancelSymbol,
    Status,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StrategyType {
    Unknown,
    Twap,
    Vwap,
    ImplementationShortfall,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OrderRequestAction {
    Unknown,
    New,
    Cancel,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResetReason {
    Unknown,
    GapDetected,
    SymbologyChanged,
    MarketDataServerDead,
}

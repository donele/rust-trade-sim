use std::collections::HashMap;

use core::{OrderRequest, OrderUpdate, Side, SymbolId};

#[derive(Debug, Default)]
pub struct OrderManager {
    updates_seen: usize,
}

impl OrderManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn on_update(&mut self, _update: OrderUpdate) {
        self.updates_seen += 1;
    }

    pub fn updates_seen(&self) -> usize {
        self.updates_seen
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub symbol: SymbolId,
    pub quantity: f64,
    pub average_price: f64,
    pub realized_pnl: f64,
}

impl Position {
    fn new(symbol: SymbolId) -> Self {
        Self {
            symbol,
            quantity: 0.0,
            average_price: 0.0,
            realized_pnl: 0.0,
        }
    }

    fn apply_fill(&mut self, side: Side, quantity: f64, price: f64) {
        let signed_quantity = match side {
            Side::Buy => quantity,
            Side::Sell => -quantity,
            Side::Unknown => return,
        };
        if quantity <= 0.0 || !quantity.is_finite() || !price.is_finite() {
            return;
        }

        if self.quantity == 0.0 || self.quantity.signum() == signed_quantity.signum() {
            let old_abs = self.quantity.abs();
            let new_abs = old_abs + quantity;
            self.average_price = if new_abs == 0.0 {
                0.0
            } else {
                (self.average_price * old_abs + price * quantity) / new_abs
            };
            self.quantity += signed_quantity;
            return;
        }

        let closing_quantity = self.quantity.abs().min(quantity);
        let pnl_per_unit = if self.quantity > 0.0 {
            price - self.average_price
        } else {
            self.average_price - price
        };
        self.realized_pnl += closing_quantity * pnl_per_unit;
        self.quantity += signed_quantity.signum() * closing_quantity;

        let opening_quantity = quantity - closing_quantity;
        if opening_quantity > 0.0 {
            self.quantity = signed_quantity.signum() * opening_quantity;
            self.average_price = price;
        } else if self.quantity == 0.0 {
            self.average_price = 0.0;
        }
    }
}

#[derive(Debug, Default)]
pub struct PositionManager {
    positions: HashMap<SymbolId, Position>,
}

impl PositionManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn on_order_update(&mut self, update: &OrderUpdate) {
        if update.filled_qty <= 0.0 {
            return;
        }
        let position = self
            .positions
            .entry(update.symbol)
            .or_insert_with(|| Position::new(update.symbol));
        position.apply_fill(update.side, update.filled_qty, update.executed_price);
    }

    pub fn get(&self, symbol: SymbolId) -> Option<&Position> {
        self.positions.get(&symbol)
    }

    pub fn quantity(&self, symbol: SymbolId) -> f64 {
        self.get(symbol).map_or(0.0, |position| position.quantity)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RiskError {
    InvalidQuantity,
    OrderQuantityExceeded { limit: f64, quantity: f64 },
    PositionLimitExceeded { limit: f64, projected_quantity: f64 },
}

#[derive(Debug, Default)]
pub struct RiskManager {
    max_order_quantity: Option<f64>,
    max_position_quantity: HashMap<SymbolId, f64>,
}

impl RiskManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_max_order_quantity(&mut self, limit: Option<f64>) {
        self.max_order_quantity = limit;
    }

    pub fn set_max_position_quantity(&mut self, symbol: SymbolId, limit: f64) {
        self.max_position_quantity.insert(symbol, limit);
    }

    pub fn check_order(
        &self,
        request: &OrderRequest,
        positions: &PositionManager,
    ) -> Result<(), RiskError> {
        if request.qty <= 0.0 || !request.qty.is_finite() {
            return Err(RiskError::InvalidQuantity);
        }
        if let Some(limit) = self.max_order_quantity {
            if request.qty > limit {
                return Err(RiskError::OrderQuantityExceeded {
                    limit,
                    quantity: request.qty,
                });
            }
        }
        if let Some(limit) = self.max_position_quantity.get(&request.symbol) {
            let signed_quantity = match request.side {
                Side::Buy => request.qty,
                Side::Sell => -request.qty,
                Side::Unknown => return Err(RiskError::InvalidQuantity),
            };
            let projected_quantity = positions.quantity(request.symbol) + signed_quantity;
            if projected_quantity.abs() > *limit {
                return Err(RiskError::PositionLimitExceeded {
                    limit: *limit,
                    projected_quantity,
                });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::{
        ClientId, OrderAction, OrderId, OrderStatus, OrderType, ParentOrderId, RejectReason,
        StrategyId, TimeInForce,
    };

    fn update(side: Side, quantity: f64, price: f64) -> OrderUpdate {
        OrderUpdate {
            ts_nanos: 0,
            symbol: SymbolId(1),
            parent_order_id: ParentOrderId(0),
            client_id: ClientId(1),
            order_id: OrderId(1),
            side,
            order_type: OrderType::Limit,
            time_in_force: TimeInForce::Gtc,
            post_only: false,
            order_price: price,
            order_qty: quantity,
            filled_qty: quantity,
            cum_qty: quantity,
            open_qty: 0.0,
            executed_price: price,
            average_fill_price: price,
            fees: 0.0,
            trade_id: 1,
            exchange_time: 0,
            strategy_id: StrategyId(1),
            order_status: OrderStatus::Filled,
            reject_reason: RejectReason::None,
        }
    }

    #[test]
    fn position_manager_tracks_average_price_and_realized_pnl() {
        let mut positions = PositionManager::new();
        positions.on_order_update(&update(Side::Buy, 2.0, 100.0));
        positions.on_order_update(&update(Side::Buy, 2.0, 110.0));
        positions.on_order_update(&update(Side::Sell, 1.0, 120.0));

        let position = positions.get(SymbolId(1)).unwrap();
        assert_eq!(position.quantity, 3.0);
        assert_eq!(position.average_price, 105.0);
        assert_eq!(position.realized_pnl, 15.0);
    }

    #[test]
    fn risk_manager_checks_order_and_position_limits() {
        let request = OrderRequest {
            ts_nanos: 0,
            symbol: SymbolId(1),
            parent_order_id: ParentOrderId(0),
            client_id: ClientId(1),
            order_id: OrderId(1),
            action: OrderAction::New,
            side: Side::Buy,
            price: 100.0,
            qty: 2.0,
            time_in_force: TimeInForce::Gtc,
            order_type: OrderType::Limit,
            strategy_id: StrategyId(1),
            exchange_time: 0,
            window: 0,
            post_only: false,
        };
        let positions = PositionManager::new();
        let mut risk = RiskManager::new();
        risk.set_max_order_quantity(Some(1.0));
        assert!(matches!(
            risk.check_order(&request, &positions),
            Err(RiskError::OrderQuantityExceeded { .. })
        ));
    }
}

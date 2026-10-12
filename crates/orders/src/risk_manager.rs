use std::collections::HashMap;

use core::{OrderRequest, Side, SymbolId};

use crate::PositionManager;

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
    use core::{ClientId, OrderAction, OrderId, OrderType, ParentOrderId, StrategyId, TimeInForce};

    #[test]
    fn checks_order_quantity_limits() {
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

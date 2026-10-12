use core::{Message, OrderRequest, OrderUpdate, StrategyId};
use orders::{OrderManager, PositionManager, RiskManager};

use crate::FarPegStrategy;

#[derive(Debug)]
pub struct StrategyHost {
    order_manager: OrderManager,
    position_manager: PositionManager,
    risk_manager: RiskManager,
    strategy: FarPegStrategy,
}

impl StrategyHost {
    pub fn new(strategy_id: StrategyId) -> Self {
        Self {
            order_manager: OrderManager::new(),
            position_manager: PositionManager::new(),
            risk_manager: RiskManager::new(),
            strategy: FarPegStrategy::new(strategy_id),
        }
    }

    pub fn on_event(&mut self, message: &Message) -> Option<OrderRequest> {
        let _ = &self.risk_manager;
        self.strategy.evaluate(message)
    }

    pub fn on_order_update(&mut self, update: OrderUpdate) {
        self.order_manager.on_update(update);
        self.position_manager.on_order_update(&update);
    }

    pub fn order_updates_seen(&self) -> usize {
        self.order_manager.updates_seen()
    }
}

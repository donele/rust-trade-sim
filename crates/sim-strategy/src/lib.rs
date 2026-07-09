use sim_core::{OrderRequest, ReplayEvent, StrategyId};
use sim_orders::{OrderManager, PositionManager, RiskManager};

#[derive(Debug)]
pub struct TradingEngine {
    strategy_id: StrategyId,
}

impl TradingEngine {
    pub fn new(strategy_id: StrategyId) -> Self {
        Self { strategy_id }
    }

    pub fn strategy_id(&self) -> StrategyId {
        self.strategy_id
    }

    pub fn evaluate(&mut self, _event: &ReplayEvent) -> Option<OrderRequest> {
        None
    }
}

#[derive(Debug)]
pub struct StrategyHost {
    order_manager: OrderManager,
    position_manager: PositionManager,
    risk_manager: RiskManager,
    trading_engine: TradingEngine,
}

impl StrategyHost {
    pub fn new(strategy_id: StrategyId) -> Self {
        Self {
            order_manager: OrderManager::new(),
            position_manager: PositionManager::new(),
            risk_manager: RiskManager::new(),
            trading_engine: TradingEngine::new(strategy_id),
        }
    }

    pub fn on_event(&mut self, event: &ReplayEvent) -> Option<OrderRequest> {
        let _ = &self.position_manager;
        let _ = &self.risk_manager;
        self.trading_engine.evaluate(event)
    }

    pub fn on_order_update(&mut self, update: sim_core::OrderUpdate) {
        self.order_manager.on_update(update);
    }

    pub fn order_updates_seen(&self) -> usize {
        self.order_manager.updates_seen()
    }
}


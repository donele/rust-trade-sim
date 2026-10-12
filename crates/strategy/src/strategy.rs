use core::{Message, OrderRequest, StrategyId};

#[derive(Debug)]
pub struct FarPegStrategy {
    strategy_id: StrategyId,
}

impl FarPegStrategy {
    pub fn new(strategy_id: StrategyId) -> Self {
        Self { strategy_id }
    }

    pub fn strategy_id(&self) -> StrategyId {
        self.strategy_id
    }

    pub fn evaluate(&mut self, _message: &Message) -> Option<OrderRequest> {
        None
    }
}

use sim_core::OrderUpdate;

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

#[derive(Debug, Default)]
pub struct PositionManager;

impl PositionManager {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug, Default)]
pub struct RiskManager;

impl RiskManager {
    pub fn new() -> Self {
        Self
    }
}


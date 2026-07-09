use sim_core::{OrderId, OrderRequest, OrderUpdate};

#[derive(Debug, Default)]
pub struct ExchangeSimulator {
    next_order_id: u64,
}

impl ExchangeSimulator {
    pub fn new() -> Self {
        Self { next_order_id: 1 }
    }

    pub fn submit(&mut self, request: OrderRequest) -> OrderUpdate {
        let order_id = OrderId(self.next_order_id);
        self.next_order_id += 1;

        OrderUpdate {
            ts_nanos: request.ts_nanos,
            order_id,
            filled_qty: 0,
            leaves_qty: request.qty,
        }
    }
}


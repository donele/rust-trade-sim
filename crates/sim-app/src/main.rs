use sim_core::{ClientId, OrderRequest, ReplayEvent, Side, StrategyId, SymbolId};
use sim_exchange::ExchangeSimulator;
use sim_feed::SimSequencer;
use sim_strategy::StrategyHost;

fn main() {
    let events = vec![ReplayEvent::OrderRequest(OrderRequest {
        ts_nanos: 0,
        client_id: ClientId(1),
        strategy_id: StrategyId(2),
        symbol: SymbolId(1),
        side: Side::Buy,
        price_ticks: 100,
        qty: 1,
    })];

    let sequencer = SimSequencer::new(events);
    let mut simulator = ExchangeSimulator::new();
    let mut strategy = StrategyHost::new(StrategyId(2));

    for event in sequencer.drain() {
        if let ReplayEvent::OrderRequest(request) = event {
            let update = simulator.submit(request);
            strategy.on_order_update(update);
        } else {
            let _ = strategy.on_event(&event);
        }
    }

    println!("trade_sim scaffold ready");
    println!("order_updates_seen={}", strategy.order_updates_seen());
    println!("next_step=implement version 1 replay, books, and exchange semantics");
}


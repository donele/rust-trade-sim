use core::{
    ClientId, Message, OrderAction, OrderId, OrderRequest, OrderType, ParentOrderId, Side,
    StrategyId, SymbolId, TimeInForce,
};
use feed::SimSequencer;
use sim_exchange::ExchangeSimulator;
use strategy::StrategyHost;

fn main() {
    let events = vec![Message::OrderRequest(OrderRequest {
        ts_nanos: 0,
        symbol: SymbolId(1),
        parent_order_id: ParentOrderId(0),
        client_id: ClientId(1),
        order_id: OrderId(0),
        action: OrderAction::New,
        side: Side::Buy,
        price: 100.0,
        qty: 1.0,
        time_in_force: TimeInForce::Gtc,
        order_type: OrderType::Limit,
        strategy_id: StrategyId(2),
        exchange_time: 0,
        window: 0,
        post_only: false,
    })];

    let sequencer = SimSequencer::new(events);
    let mut simulator = ExchangeSimulator::new();
    let mut strategy = StrategyHost::new(StrategyId(2));

    for event in sequencer.drain() {
        if let Message::OrderRequest(request) = event {
            let update = simulator.submit(request);
            strategy.on_order_update(update);
        } else {
            let _ = strategy.on_event(&event);
        }
    }

    println!("trade_sim scaffold ready");
    println!("order_updates_seen={}", strategy.order_updates_seen());
    println!("next_step=implement historical feeds and deterministic sequencing");
}

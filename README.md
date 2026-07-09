# rust-trade-sim

Rust implementation of the trading simulator being developed in Java and already working in C++.

The functional reference runtime is:

`SimSequencer -> ExchangeSimulator -> StrategyHost -> TradingEngine`

This repo currently contains the version-1 scaffold for that architecture:

- Cargo workspace with crate boundaries matching the design plan
- minimal shared event and identifier model
- placeholder sequencer, exchange simulator, order managers, and strategy host
- `trade_sim` binary that exercises the scaffold end to end

For the design plan, see [DESIGN_PLAN.md](./DESIGN_PLAN.md).

## Version 1 target

Version 1 is a deterministic historical simulator with a clear vertical slice.

It aims to cover:

- core market and order event modeling
- deterministic replay and sequencing
- exchange simulation
- order, position, and risk state
- strategy host and a simplified trading engine
- one or two trader implementations
- local configuration loading for offline runs

Version 1 does not aim to cover:

- live shared-memory playback
- production order placer integration
- full C++ alpha graph parity
- aggressive low-level optimization

## Repository layout

- `crates/sim-core`
  - shared identifiers, time aliases, and event types
- `crates/sim-feed`
  - replay source and sequencing layer
- `crates/sim-exchange`
  - exchange simulator and simulated order lifecycle
- `crates/sim-orders`
  - order, position, and risk managers
- `crates/sim-strategy`
  - strategy host and trading engine layer
- `crates/sim-app`
  - `trade_sim` binary

## Prerequisites

You need a Rust toolchain with `cargo`.

Recommended installation:

```bash
curl https://sh.rustup.rs -sSf | sh
source "$HOME/.cargo/env"
rustc --version
cargo --version
```

This scaffold uses only the Rust standard library, so it does not require external crate downloads
 beyond the toolchain itself.

## Build

From the repository root:

```bash
cargo build
```

For an optimized build:

```bash
cargo build --release
```

## Run

Run the scaffold binary from the repository root:

```bash
cargo run -p sim-app --bin trade_sim
```

Expected output is a short scaffold status message including:

- `trade_sim scaffold ready`
- `order_updates_seen=1`

For an optimized run:

```bash
cargo run --release -p sim-app --bin trade_sim
```

## Current status

This is a scaffold, not yet the full simulator.

Implemented today:

- workspace structure
- minimal compile-target types
- placeholder binary that exercises the intended top-level flow

Still to implement for version 1:

- real market-data event set
- order book and per-symbol state
- merged replay sequencer
- exchange fill and cancel semantics
- order / position / risk lifecycle logic
- strategy dispatch and trader behavior
- config-driven input loading

## Next development steps

The intended implementation order is:

1. expand `sim-core` into the stable message and identifier model
2. implement books and symbol state
3. implement deterministic replay sequencing
4. implement exchange simulation semantics
5. implement order, position, and risk managers
6. implement strategy host and first working trader

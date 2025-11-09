    @@@@@@@   @@@@@@@   @@@@@@
    @@@@@@@@  @@@@@@@  @@@@@@@
    @@!  @@@    @@!    !@@
    !@!  @!@    !@!    !@!
    @!@!!@!     @!!    !!@@!!
    !!@!@!      !!!     !!@!!!
    !!: :!!     !!:         !:!
    :!:  !:!    :!:        !:!
    ::   :::     ::    :::: ::
     :   : :     :     :: : :

(Rust Trading System)

The idea is to build up HFT/MFT backtesting/trading engine on top of rust-orderbook with
an emphasis on high performance and strategy research robustness.

RTS Modules:

1. rust-orderbook (highly performant L2/L3 orderbook impl)
2. trade-rs (latency & backtest PnL)
3. bond-rs (bond pricing lib) "The name's bond, trades bonds"
4. GUI/TUI for backtesting/market data viz and Comand&Control
5. butte-rs spread calcualtion and visualization

# Rust Orderbook.

Limit Orderbook written in Rust.

Run toy strategy:

```
cargo run --release --example strategy_flow
```

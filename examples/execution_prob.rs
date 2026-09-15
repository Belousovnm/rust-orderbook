use clap::Parser;
use orderbook::{
    backtest::FixPriceStrategy, engine::OrderBook, engine::TradingAccount,
    experiments::execution_flow, management::OrderManagementSystem,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// path to orderbook files
    #[arg(short = 'x', long)]
    ob_path: String,

    /// path to orders files
    #[arg(short, long)]
    orders_path: String,
}

fn main() {
    let args = Args::parse();
    let mut ob = OrderBook::default();
    let mut strat = FixPriceStrategy {
        buy_tick_criterion: Some(-2),
        sell_tick_criterion: None,
        qty: 1,
        ..Default::default()
    };

    let money_account = TradingAccount::new(0.0);
    let mut oms = OrderManagementSystem::new(&mut strat, money_account);

    execution_flow(
        &mut oms,
        &mut ob,
        args.ob_path.as_str(),
        args.orders_path.as_str(),
    );
}

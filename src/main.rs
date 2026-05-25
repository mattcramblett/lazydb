use clap::Parser;
use cli::Cli;

use crate::app::App;

mod action;
mod app;
mod app_event;
mod cli;
mod components;
mod config;
mod database;
mod errors;
mod logging;
mod render_plan;
mod tui;
mod widgets;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    crate::errors::init()?;
    crate::logging::init()?;

    let args = Cli::parse();
    let mut app = App::new(args.tick_rate, args.frame_rate, args.debug)?;
    app.run().await?;
    Ok(())
}

use clap::Parser;
use cli::Cli;

use crate::app::App;

mod action;
mod app_event;
mod app;
mod cli;
mod components;
mod render_plan;
mod config;
mod errors;
mod logging;
mod tui;
mod database;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    crate::errors::init()?;
    crate::logging::init()?;

    let args = Cli::parse();
    let mut app = App::new(args.tick_rate, args.frame_rate)?;
    app.run().await?;
    Ok(())
}

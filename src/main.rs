#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::unwrap_used,
    clippy::expect_used
)]
mod bot;
mod config;
mod wisps;

pub use bot::Bot;

use color_eyre::eyre::Result;
use tracing::{instrument, warn};

#[tokio::main]
#[instrument]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let token = config::discord_token()?;
    bot::run(&token).await
}

mod bot;
mod commands;
mod serenity_ext;

use std::env;

use color_eyre::eyre::{eyre, Result};
use tracing::{instrument, warn};

#[tokio::main]
#[instrument]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let token = env::var("DISCORD_TOKEN").map_err(|_| eyre!("$DISCORD_TOKEN not set"))?;
    bot::run(&token).await
}

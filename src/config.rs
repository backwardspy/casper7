use std::env;

use color_eyre::{eyre::eyre, Result};
use serenity::model::prelude::GuildId;
use tracing::{info, instrument, warn};

pub fn discord_token() -> Result<String> {
    env::var("DISCORD_TOKEN").map_err(|_| eyre!("$DISCORD_TOKEN not set"))
}

pub fn database_path() -> String {
    env::var("DATABASE_PATH").unwrap_or_else(|_| "casper.db".to_owned())
}

#[instrument]
pub fn testing_guild() -> Option<GuildId> {
    let guild_id = match env::var("TESTING_GUILD") {
        Ok(guild_id) => guild_id,
        Err(e) => {
            info!("$TESTING_GUILD not set ({e})");
            return None;
        }
    };

    match guild_id.parse::<u64>() {
        Ok(guild_id) => Some(GuildId(guild_id)),
        Err(e) => {
            warn!("Ignoring $TESTING_GUILD: {e}");
            None
        }
    }
}

pub fn meatball_assignment_schedule() -> String {
    env::var("MEATBALL_ASSIGNMENT_SCHEDULE").unwrap_or_else(|_| "*/10 * * * * *".to_owned())
}

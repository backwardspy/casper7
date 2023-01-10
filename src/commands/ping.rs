use color_eyre::Result;
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::interaction::application_command::CommandDataOption,
};

use crate::bot::CommandContext;

use super::cmd;

const PING: &str = "ping";

pub(crate) async fn dispatch(
    name: &str,
    _options: &[CommandDataOption],
    _context: &CommandContext<'_>,
) -> Result<Option<String>> {
    Ok(match name {
        PING => Some("pong".to_owned()),
        _ => None,
    })
}

pub(crate) fn get_commands() -> Vec<CreateApplicationCommand> {
    vec![cmd(PING, "Check bot liveness.")]
}

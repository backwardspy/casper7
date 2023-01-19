use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::interaction::application_command::CommandDataOption,
};

use crate::bot::CommandContext;

use super::cmd;

const PING: &str = "ping";

pub fn dispatch(
    name: &str,
    _options: &[CommandDataOption],
    _context: &CommandContext<'_>,
) -> Option<String> {
    match name {
        PING => Some("pong".to_owned()),
        _ => None,
    }
}

pub fn get_commands() -> Vec<CreateApplicationCommand> {
    vec![cmd(PING, "Check bot liveness.")]
}

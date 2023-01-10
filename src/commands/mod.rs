use color_eyre::Result;
use serenity::{
    builder::{
        CreateApplicationCommand, CreateApplicationCommandOption, CreateApplicationCommands,
    },
    model::prelude::{
        command::CommandOptionType, interaction::application_command::CommandDataOption,
    },
};
use tracing::{info, instrument, warn};

use crate::bot::CommandContext;

mod meatball;
mod ping;

/// Utility function for quickly creating a `CreateApplicationCommand`.
fn cmd(name: &str, description: &str) -> CreateApplicationCommand {
    let mut cmd = CreateApplicationCommand::default();
    cmd.name(name).description(description);
    cmd
}

/// Utility function for quickly creating a `CreateApplicationCommand`. that takes a closure for
/// customising the command.
fn cmd_custom(
    name: &str,
    description: &str,
    customize: fn(&mut CreateApplicationCommand) -> &mut CreateApplicationCommand,
) -> CreateApplicationCommand {
    let mut cmd = CreateApplicationCommand::default();
    cmd.name(name).description(description);
    customize(&mut cmd);
    cmd
}

/// Utility function for quickly creating a `CreateApplicationCommandOption`.
fn opt(
    name: &str,
    description: &str,
    kind: CommandOptionType,
    required: bool,
) -> CreateApplicationCommandOption {
    let mut opt = CreateApplicationCommandOption::default();
    opt.name(name)
        .description(description)
        .kind(kind)
        .required(required);
    opt
}

/// Register all slash commands.
pub(crate) fn register_all(
    setter: &mut CreateApplicationCommands,
) -> &mut CreateApplicationCommands {
    setter
        .set_application_commands(ping::get_commands())
        .set_application_commands(meatball::get_commands())
}

#[instrument(skip(context))]
pub(crate) async fn dispatch(
    name: &str,
    options: &[CommandDataOption],
    context: CommandContext<'_>,
) -> Result<String> {
    info!("Starting dispatch");

    if let Some(content) = ping::dispatch(name, options, &context).await? {
        return Ok(content);
    }

    if let Some(content) = meatball::dispatch(name, options, &context).await? {
        return Ok(content);
    }

    warn!("Command '{name}' has no dispatch. Returning placeholder response.");
    Ok("This command has not yet been implemented!".to_owned())
}

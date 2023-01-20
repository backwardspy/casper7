use color_eyre::{eyre::eyre, Result};
use serenity::{
    async_trait,
    model::prelude::{
        command::Command,
        interaction::{
            application_command::ApplicationCommandInteraction, Interaction,
            InteractionResponseType,
        },
        GuildId, Message, Ready,
    },
    prelude::*,
    Client,
};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    SqlitePool,
};
use tokio_cron_scheduler::JobScheduler;
use tracing::{error, info, instrument};

use crate::{commands, config, jobs, listeners};

#[instrument(skip(ctx))]
async fn global_command_creator(ctx: &Context) {
    match Command::set_global_application_commands(&ctx.http, commands::register_all).await {
        Ok(commands) => {
            for command in &commands {
                info!(name = command.name, description = command.description);
            }
        }
        Err(e) => error!("Failed to register global commands: {e}"),
    }
}

#[instrument(skip(ctx))]
async fn guild_command_creator(ctx: &Context, guild_id: GuildId) {
    match GuildId::set_application_commands(&guild_id, &ctx.http, commands::register_all).await {
        Ok(commands) => {
            for command in &commands {
                info!(name = command.name, description = command.description);
            }
        }
        Err(e) => error!("Failed to register guild commands: {e}"),
    };
}

struct Bot {
    db: SqlitePool,
    scheduler: JobScheduler,
}

pub struct CommandContext<'a> {
    pub(crate) db: &'a SqlitePool,
    pub(crate) interaction: &'a ApplicationCommandInteraction,
}

#[derive(Clone)]
pub struct JobContext {
    pub(crate) ctx: Context,
    pub(crate) db: SqlitePool,
}

impl Bot {
    async fn spawn_scheduler(&self, ctx: Context) -> Result<()> {
        info!("Spawning scheduler");

        let job_ctx = JobContext {
            ctx,
            db: self.db.clone(),
        };
        for job in jobs::all(job_ctx)? {
            self.scheduler.add(job).await?;
        }

        self.scheduler.start().await?;

        Ok(())
    }
}

#[async_trait]
impl EventHandler for Bot {
    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        if let Err(e) = self.spawn_scheduler(ctx).await {
            error!("Failed to setup scheduler: {e}");
        }
    }

    async fn message(&self, ctx: Context, message: Message) {
        if let Err(e) = listeners::dispatch(&ctx, message).await {
            error!("Failure in message listeners: {e}");
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} connected successfully", ready.user.name);

        if let Some(guild_id) = config::testing_guild() {
            info!("Setting up slash commands for testing guild {guild_id}");
            guild_command_creator(&ctx, guild_id).await;
        } else {
            info!("Setting up global slash commands");
            global_command_creator(&ctx).await;
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let cmd_ctx = CommandContext {
                db: &self.db,
                interaction: &command,
            };

            match commands::dispatch(&command.data.name, &command.data.options, cmd_ctx).await {
                Ok(content) => {
                    if let Err(e) = command
                        .create_interaction_response(&ctx.http, |response| {
                            response
                                .kind(InteractionResponseType::ChannelMessageWithSource)
                                .interaction_response_data(|message| message.content(content))
                        })
                        .await
                    {
                        error!("Failed to respond to slash command: {e}");
                    }
                }
                Err(e) => error!("Failed to dispatch command: {e}"),
            }
        }
    }
}

pub async fn run(token: &str) -> Result<()> {
    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(config::database_path())
                .create_if_missing(true),
        )
        .await?;

    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(Bot {
            db,
            scheduler: JobScheduler::new().await?,
        })
        .await
        .map_err(|e| eyre!("Failed to create client: {e}"))?;

    client
        .start()
        .await
        .map_err(|e| eyre!("Failed to start client: {e}"))?;

    Ok(())
}

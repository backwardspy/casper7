use std::{env, time::Duration};

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
use tracing::{error, info, instrument, warn};

use crate::{commands, jobs, listeners};

#[instrument]
fn use_testing_guild() -> Option<GuildId> {
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
}

pub(crate) struct CommandContext<'a> {
    pub(crate) db: &'a SqlitePool,
    pub(crate) interaction: &'a ApplicationCommandInteraction,
}

#[derive(Clone)]
pub(crate) struct JobContext {
    pub(crate) ctx: Context,
    pub(crate) db: SqlitePool,
}

async fn scheduler_thread(ctx: Context, db: SqlitePool) -> Result<()> {
    let scheduler = JobScheduler::new().await?;

    let job_ctx = JobContext { ctx, db };

    for job in jobs::all(job_ctx)?.into_iter() {
        scheduler.add(job).await?;
    }

    scheduler.start().await?;

    loop {
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

impl Bot {
    async fn spawn_scheduler(&self, ctx: Context) -> Result<()> {
        info!("Spawning scheduler");

        let pool = self.db.clone();

        std::thread::Builder::new()
            .name("schedule thread".to_string())
            .spawn(move || {
                tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .expect("Build scheduler runtime failed")
                    .block_on(scheduler_thread(ctx, pool))
                    .expect("Scheduler thread crashed");
            })?;

        Ok(())
    }
}

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} connected successfully", ready.user.name);

        if let Some(guild_id) = use_testing_guild() {
            info!("Setting up slash commands for testing guild {guild_id}");
            guild_command_creator(&ctx, guild_id).await;
        } else {
            info!("Setting up global slash commands");
            global_command_creator(&ctx).await;
        }
    }

    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        if let Err(e) = self.spawn_scheduler(ctx.clone()).await {
            error!("Failed to setup scheduler: {e}");
        }
    }

    async fn message(&self, ctx: Context, message: Message) {
        listeners::dispatch(ctx, message);
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let context = CommandContext {
                db: &self.db,
                interaction: &command,
            };

            match commands::dispatch(&command.data.name, &command.data.options, context).await {
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
                .filename("casper.db")
                .create_if_missing(true),
        )
        .await?;

    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(Bot { db })
        .await
        .map_err(|e| eyre!("Failed to create client: {e}"))?;

    client
        .start()
        .await
        .map_err(|e| eyre!("Failed to start client: {e}"))?;

    Ok(())
}

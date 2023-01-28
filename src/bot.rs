use poise::serenity_prelude as serenity;
use std::future::Future;

use color_eyre::{
    eyre::{eyre, ErrReport},
    Result,
};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    SqlitePool,
};
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info};

use crate::{config, wisps};

pub struct Bot {
    pub db: SqlitePool,
    scheduler: JobScheduler,
}

pub type CommandContext<'a> = poise::Context<'a, Bot, ErrReport>;

#[derive(Clone)]
pub struct JobContext {
    pub(crate) ctx: serenity::Context,
    pub(crate) db: SqlitePool,
}

fn make_job<F, Fut>(name: &str, schedule: &str, callback: F, ctx: JobContext) -> Result<Job>
where
    F: Send + Sync + Copy + FnOnce(JobContext) -> Fut + 'static,
    Fut: Send + Future<Output = Result<()>>,
{
    let job_name = name.to_owned();
    Job::new_async(schedule, move |_uuid, _lock| {
        let job_name = job_name.clone();
        let ctx = ctx.clone();
        Box::pin(async move {
            match callback(ctx).await {
                Ok(()) => {
                    info!("Job {job_name} completed successfully.");
                }
                Err(e) => {
                    error!("Job {job_name} failed: {e}");
                }
            }
        })
    })
    .map_err(|e| eyre!("failed to create job {name}: {e}"))
}

impl Bot {
    async fn spawn_scheduler(&self, ctx: serenity::Context) -> Result<()> {
        info!("Spawning scheduler");

        let job_ctx = JobContext {
            ctx,
            db: self.db.clone(),
        };

        self.scheduler
            .add(make_job(
                "meatball::update_role_assignments",
                &config::meatball_assignment_schedule(),
                wisps::meatball::jobs::update_role_assignments,
                job_ctx.clone(),
            )?)
            .await?;

        self.scheduler.start().await?;

        Ok(())
    }
}

async fn event_handler(
    ctx: &serenity::Context,
    event: &poise::Event<'_>,
    _framework: poise::FrameworkContext<'_, Bot, color_eyre::eyre::ErrReport>,
    bot: &Bot,
) -> Result<(), color_eyre::eyre::ErrReport> {
    match event {
        poise::Event::CacheReady { guilds: _ } => {
            if let Err(e) = bot.spawn_scheduler(ctx.clone()).await {
                error!("Failed to setup scheduler: {e}");
            }
        }
        poise::Event::Message {
            new_message: message,
        } => {
            if let Err(e) = wisps::wordle::listeners::dispatch(ctx, message).await {
                error!("Failure in message listeners: {e}");
            }
        }
        poise::Event::Ready {
            data_about_bot: ready,
        } => {
            info!("{} connected successfully", ready.user.name);
        }
        _ => {}
    }
    Ok(())
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

    let bot = Bot {
        db,
        scheduler: JobScheduler::new().await?,
    };

    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: wisps::commands(),
            event_handler: |ctx, event, framework, bot| {
                Box::pin(event_handler(ctx, event, framework, bot))
            },
            ..Default::default()
        })
        .token(token)
        .intents(intents)
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                if let Some(guild_id) = config::testing_guild() {
                    info!("Setting up slash commands for testing guild {guild_id}");
                    poise::builtins::register_in_guild(
                        ctx,
                        &framework.options().commands,
                        guild_id,
                    )
                    .await?;
                } else {
                    info!("Setting up global slash commands");
                    poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                }
                Ok(bot)
            })
        });

    framework.run().await?;

    Ok(())
}

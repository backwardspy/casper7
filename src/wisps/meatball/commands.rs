use chrono::{Datelike, TimeZone};
use color_eyre::{eyre::eyre, Result};
use itertools::{iproduct, Itertools};
use poise::serenity_prelude as serenity;
use tracing::warn;

use crate::bot::CommandContext;

// January 2nd, 2023
const DATE_FORMAT: &str = "%B %d, %Y";
// January 2nd
const DAY_MONTH_FORMAT: &str = "%B %d";

#[poise::command(
    slash_command,
    subcommands("lookup", "next", "save", "forget", "channel", "role")
)]
#[allow(clippy::unused_async)]
pub async fn meatball(_ctx: CommandContext<'_>) -> Result<()> {
    Ok(())
}

/// Find a user's meatball day, if they have saved one.
#[poise::command(slash_command)]
pub async fn lookup(
    ctx: CommandContext<'_>,
    #[description = "The user to lookup (defaults to you)"] user: Option<serenity::User>,
) -> Result<()> {
    let guild = ctx.guild().ok_or(eyre!("Command run without guild"))?;
    let user_id = user.map_or_else(|| ctx.author().id, |u| u.id);

    let row: Option<(u32, u32)> = sqlx::query_as(include_str!("queries/meatball-lookup.sql"))
        .bind(guild.id.to_string())
        .bind(user_id.to_string())
        .fetch_optional(&ctx.data().db)
        .await?;

    let response = if let Some((month, day)) = row {
        let date = chrono::Utc
            .with_ymd_and_hms(2000, month, day, 0, 0, 0)
            .earliest()
            .ok_or(eyre!("Failed to create dummy date for lookup return"))?;
        format!(
            "{}'s meatball day is on {}",
            serenity::Mention::from(user_id),
            date.format(DAY_MONTH_FORMAT)
        )
    } else {
        format!(
            "I don't have {}'s meatball day registered!",
            serenity::Mention::from(user_id)
        )
    };

    ctx.say(response).await?;

    Ok(())
}

/// Find the next occurring meatball day.
#[poise::command(slash_command)]
pub async fn next(ctx: CommandContext<'_>) -> Result<()> {
    let guild = ctx.guild().ok_or(eyre!("Command run without guild"))?;

    let now = chrono::Utc::now();
    let years = [now.year(), now.year() + 1].into_iter();

    let rows: Vec<(String, u32, u32)> = sqlx::query_as(include_str!("queries/meatball-next.sql"))
        .bind(guild.id.to_string())
        .fetch_all(&ctx.data().db)
        .await?;

    let mut meatball_days = iproduct!(rows, years)
        .map(|((user, month, day), year)| {
            let date = chrono::Utc
                .with_ymd_and_hms(year, month, day, 0, 0, 0)
                .earliest();

            if date.is_none() {
                warn!("Skipping {user}'s invalid meatball day: {month}/{day}");
            }

            (user, date)
        })
        .filter_map(|(user, date)| date.map(|date| (user, date)))
        .sorted_by(|(_, a), (_, b)| Ord::cmp(a, b));

    let response = if let Some((user, date)) = meatball_days.find(|(_, date)| *date > now) {
        format!(
            "The next meatball day is {}'s on {}! :alarm_clock:",
            serenity::Mention::from(serenity::UserId(user.parse()?)),
            date.format(DATE_FORMAT)
        )
    } else {
        "I have no meatball days saved!".to_owned()
    };

    ctx.say(response).await?;

    Ok(())
}

const fn days_in_month(month: i64) -> i64 {
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31][month as usize - 1]
}

/// Save your meatball day.
#[poise::command(slash_command)]
pub async fn save(
    ctx: CommandContext<'_>,
    #[description = "The month of your meatball day"] month: i64,
    #[description = "The day of your meatball day"] day: i64,
) -> Result<()> {
    let guild = ctx.guild().ok_or(eyre!("Command run without guild"))?;

    if !(1..=12).contains(&month) {
        ctx.say("That's not a real month... :thinking:").await?;
        return Ok(());
    }

    let num_days = days_in_month(month);
    if day < 1 || day > num_days {
        ctx.say("That's not a real day of the month... :thinking:")
            .await?;
        return Ok(());
    }

    sqlx::query(include_str!("queries/meatball-save.sql"))
        .bind(guild.id.to_string())
        .bind(ctx.author().id.to_string())
        .bind(month)
        .bind(day)
        .execute(&ctx.data().db)
        .await?;

    ctx.say("I have registered your meatball day! :calendar:")
        .await?;

    Ok(())
}

/// Remove your meatball day.
#[poise::command(slash_command)]
pub async fn forget(ctx: CommandContext<'_>) -> Result<()> {
    let guild = ctx.guild().ok_or(eyre!("Command run without guild"))?;

    sqlx::query(include_str!("queries/meatball-forget.sql"))
        .bind(guild.id.to_string())
        .bind(ctx.author().id.to_string())
        .execute(&ctx.data().db)
        .await?;

    ctx.say("I have removed your meatball day from the database. :boom:")
        .await?;

    Ok(())
}

/// Set the channel to use for announcements.
#[poise::command(slash_command, required_permissions = "ADMINISTRATOR")]
pub async fn channel(
    ctx: CommandContext<'_>,
    #[description = "The channel to use"] channel: serenity::GuildChannel,
) -> Result<()> {
    let guild = ctx.guild().ok_or(eyre!("Command run without guild"))?;

    if channel.kind != serenity::ChannelType::Text {
        ctx.say("I can only announce in normal text channels.")
            .await?;
        return Ok(());
    }

    sqlx::query(include_str!("queries/meatball-channel.sql"))
        .bind(guild.id.to_string())
        .bind(channel.id.to_string())
        .execute(&ctx.data().db)
        .await?;

    ctx.say(format!(
        "I have set the announcements channel to {}",
        serenity::Mention::from(channel.id)
    ))
    .await?;

    Ok(())
}

/// Set the role to assign on meatball day.
#[poise::command(slash_command, required_permissions = "ADMINISTRATOR")]
pub async fn role(
    ctx: CommandContext<'_>,
    #[description = "The role to assign"] role: serenity::Role,
) -> Result<()> {
    let guild = ctx.guild().ok_or(eyre!("Command run without guild"))?;

    sqlx::query(include_str!("queries/meatball-role.sql"))
        .bind(guild.id.to_string())
        .bind(role.id.to_string())
        .execute(&ctx.data().db)
        .await?;

    ctx.say(format!(
        "I have set the meatball day role to {}",
        serenity::Mention::from(role.id)
    ))
    .await?;

    Ok(())
}

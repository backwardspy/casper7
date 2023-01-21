use chrono::{Datelike, TimeZone};
use color_eyre::{eyre::eyre, Result};
use itertools::{iproduct, Itertools};
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        command::CommandOptionType, interaction::application_command::CommandDataOption, Mention,
        UserId,
    },
};
use tracing::warn;

use crate::{
    bot::CommandContext,
    serenity_ext::{get_channel_option, get_integer_option, get_role_option, get_user_option},
};

use super::{cmd, cmd_custom, opt};

// January 2nd, 2023
const DATE_FORMAT: &str = "%B %d, %Y";
// January 2nd
const DAY_MONTH_FORMAT: &str = "%B %d";

const LOOKUP: &str = "meatball-lookup";
const NEXT: &str = "meatball-next";
const SAVE: &str = "meatball-save";
const FORGET: &str = "meatball-forget";
const CHANNEL: &str = "meatball-channel";
const ROLE: &str = "meatball-role";

pub fn get_commands() -> Vec<CreateApplicationCommand> {
    vec![
        cmd_custom(LOOKUP, "Find a user's meatball day", |cmd| {
            cmd.add_option(opt(
                "user",
                "The user to lookup (defaults to you)",
                CommandOptionType::User,
                false,
            ))
        }),
        cmd(NEXT, "Find the next occurring meatball day."),
        cmd_custom(SAVE, "Add your meatball day to the database", |cmd| {
            cmd.add_option(opt(
                "month",
                "The month of your meatball day.",
                CommandOptionType::Integer,
                true,
            ))
            .add_option(opt(
                "day",
                "The day of your meatball day",
                CommandOptionType::Integer,
                true,
            ))
        }),
        cmd(FORGET, "Remove your meatball day from the database."),
        cmd_custom(
            CHANNEL,
            "Set the channel to use for announcements.",
            |cmd| {
                cmd.add_option(opt(
                    "channel",
                    "The channel to use for announcements",
                    CommandOptionType::Channel,
                    true,
                ))
            },
        ),
        cmd_custom(ROLE, "Set the role to assign on meatball day.", |cmd| {
            cmd.add_option(opt(
                "role",
                "The role to assign on meatball day.",
                CommandOptionType::Role,
                true,
            ))
        }),
    ]
}

pub async fn dispatch(
    name: &str,
    options: &[CommandDataOption],
    context: &CommandContext<'_>,
) -> Result<Option<String>> {
    Ok(match name {
        LOOKUP => Some(lookup(options, context).await?),
        NEXT => Some(next(options, context).await?),
        SAVE => Some(save(options, context).await?),
        FORGET => Some(forget(options, context).await?),
        CHANNEL => Some(channel(options, context).await?),
        ROLE => Some(role(options, context).await?),
        _ => None,
    })
}

async fn lookup(options: &[CommandDataOption], context: &CommandContext<'_>) -> Result<String> {
    let user = get_user_option(options, 0).unwrap_or(&context.interaction.user);
    let guild = context
        .interaction
        .guild_id
        .ok_or(eyre!("Command run without guild"))?;

    let row: Option<(u32, u32)> = sqlx::query_as(include_str!("queries/meatball-lookup.sql"))
        .bind(guild.to_string())
        .bind(user.id.to_string())
        .fetch_optional(context.db)
        .await?;

    Ok(if let Some((month, day)) = row {
        let date = chrono::Utc
            .with_ymd_and_hms(2000, month, day, 0, 0, 0)
            .earliest()
            .ok_or(eyre!("Failed to create dummy date for lookup return"))?;
        format!(
            "{}'s meatball day is on {}",
            Mention::from(user.id),
            date.format(DAY_MONTH_FORMAT)
        )
    } else {
        format!(
            "I don't have {}'s meatball day registered!",
            Mention::from(user.id)
        )
    })
}

async fn next(_options: &[CommandDataOption], context: &CommandContext<'_>) -> Result<String> {
    let guild = context
        .interaction
        .guild_id
        .ok_or(eyre!("Command run without guild"))?;

    let now = chrono::Utc::now();
    let years = [now.year(), now.year() + 1].into_iter();

    let rows: Vec<(String, u32, u32)> = sqlx::query_as(include_str!("queries/meatball-next.sql"))
        .bind(guild.to_string())
        .fetch_all(context.db)
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

    Ok(
        if let Some((user, date)) = meatball_days.find(|(_, date)| *date > now) {
            format!(
                "The next meatball day is {}'s on {}! :alarm_clock:",
                Mention::from(UserId(user.parse()?)),
                date.format(DATE_FORMAT)
            )
        } else {
            "I have no meatball days saved!".to_owned()
        },
    )
}

const fn days_in_month(month: i64) -> i64 {
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31][month as usize - 1]
}

async fn save(options: &[CommandDataOption], context: &CommandContext<'_>) -> Result<String> {
    let guild = context
        .interaction
        .guild_id
        .ok_or(eyre!("Command run without guild"))?;

    let month =
        get_integer_option(options, 0).ok_or(eyre!("Expected integer month as option #0"))?;
    let day = get_integer_option(options, 1).ok_or(eyre!("Expected integer day as option #1"))?;

    if !(1..=12).contains(&month) {
        return Ok("That's not a real month... :thinking:".to_owned());
    }

    let num_days = days_in_month(month);
    if day < 1 || day > num_days {
        return Ok("That's not a real day of the month... :thinking:".to_owned());
    }

    sqlx::query(include_str!("queries/meatball-save.sql"))
        .bind(guild.to_string())
        .bind(context.interaction.user.id.to_string())
        .bind(month)
        .bind(day)
        .execute(context.db)
        .await?;

    Ok("I have registered your meatball day! :calendar:".to_owned())
}

async fn forget(_options: &[CommandDataOption], context: &CommandContext<'_>) -> Result<String> {
    let guild = context
        .interaction
        .guild_id
        .ok_or(eyre!("Command run without guild"))?;

    sqlx::query(include_str!("queries/meatball-forget.sql"))
        .bind(guild.to_string())
        .bind(context.interaction.user.id.to_string())
        .execute(context.db)
        .await?;

    Ok("I have removed your meatball day from the database. :boom:".to_owned())
}

async fn channel(options: &[CommandDataOption], context: &CommandContext<'_>) -> Result<String> {
    let guild = context
        .interaction
        .guild_id
        .ok_or(eyre!("Command run without guild"))?;

    let channel = get_channel_option(options, 0).ok_or(eyre!("Expected channel as option 0"))?;

    sqlx::query(include_str!("queries/meatball-channel.sql"))
        .bind(guild.to_string())
        .bind(channel.id.to_string())
        .execute(context.db)
        .await?;

    Ok(format!(
        "I have set the announcements channel to {}",
        Mention::from(channel.id)
    ))
}

async fn role(options: &[CommandDataOption], context: &CommandContext<'_>) -> Result<String> {
    let guild = context
        .interaction
        .guild_id
        .ok_or(eyre!("Command run without guild"))?;

    let role = get_role_option(options, 0).ok_or(eyre!("Expected role as option 0"))?;

    sqlx::query(include_str!("queries/meatball-role.sql"))
        .bind(guild.to_string())
        .bind(role.id.to_string())
        .execute(context.db)
        .await?;

    Ok(format!(
        "I have set the meatball day role to {}",
        Mention::from(role.id)
    ))
}

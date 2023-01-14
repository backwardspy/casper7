use chrono::Datelike;
use color_eyre::{eyre::eyre, Result};
use serenity::{
    model::prelude::{ChannelId, GuildId, RoleId, UserId},
    prelude::Mentionable,
};
use sqlx::SqlitePool;
use tracing::info;

use crate::bot::JobContext;

pub(crate) async fn update_role_assignments(ctx: JobContext) -> Result<()> {
    remove_expired_assignments(&ctx).await?;
    add_pending_assignments(&ctx).await?;
    Ok(())
}

async fn get_guild_role(guild: GuildId, pool: &SqlitePool) -> Result<RoleId> {
    let result: (String,) = sqlx::query_as(include_str!("queries/get_guild_meatball_role.sql"))
        .bind(guild.to_string())
        .fetch_one(pool)
        .await
        .map_err(|e| {
            eyre!("failed to get role for guild {guild}. it might not be set yet! ({e})")
        })?;

    Ok(RoleId(result.0.parse()?))
}

async fn get_guild_channel(guild: GuildId, pool: &SqlitePool) -> Result<ChannelId> {
    let result: (String,) = sqlx::query_as(include_str!("queries/get_guild_meatball_channel.sql"))
        .bind(guild.to_string())
        .fetch_one(pool)
        .await
        .map_err(|e| {
            eyre!("failed to get role for guild {guild}. it might not be set yet! ({e})")
        })?;

    Ok(ChannelId(result.0.parse()?))
}

async fn add_pending_assignments(ctx: &JobContext) -> Result<()> {
    let pending = get_pending_assignments(&ctx.db).await?;

    for (guild, user) in pending.into_iter() {
        add_pending_assignment(guild, user, ctx).await?;
    }

    Ok(())
}

async fn add_pending_assignment(guild: GuildId, user: UserId, ctx: &JobContext) -> Result<()> {
    // if we fail to add the assignment, we can roll back the transaction
    // and try again later.
    let tx = ctx.db.begin().await?;

    let role = get_guild_role(guild, &ctx.db).await?;
    let mut member = guild.member(&ctx.ctx.http, user).await?;

    info!(
        "Adding role '{}' to member '{}' of guild '{}'",
        role,
        member.display_name(),
        guild.name(&ctx.ctx.cache).unwrap_or(guild.to_string())
    );
    member.add_role(&ctx.ctx.http, role).await?;

    let channel = get_guild_channel(guild, &ctx.db).await?;
    info!(
        "Notifying channel '{}'",
        channel
            .name(&ctx.ctx.cache)
            .await
            .unwrap_or(channel.to_string())
    );
    channel
        .send_message(&ctx.ctx.http, |message| {
            message.content(format!(
                "It's {}'s meatball day! :partying_face::tada:",
                member.mention()
            ))
        })
        .await?;

    info!("Role added successfully, adding role assignment to DB");
    create_assignment(guild, user, &ctx.db).await?;

    tx.commit().await?;

    Ok(())
}

async fn get_pending_assignments(pool: &SqlitePool) -> Result<Vec<(GuildId, UserId)>> {
    let today = chrono::Utc::now();
    let rows: Vec<(String, String)> =
        sqlx::query_as(include_str!("queries/get_pending_meatball_assignments.sql"))
            .bind(today.month())
            .bind(today.day())
            .fetch_all(pool)
            .await?;

    let mut new = vec![];
    for (guild, user) in rows.into_iter() {
        new.push((GuildId(guild.parse()?), UserId(user.parse()?)));
    }

    Ok(new)
}

async fn remove_expired_assignments(ctx: &JobContext) -> Result<()> {
    let expired = get_expired_assignments(&ctx.db).await?;

    for (guild, user) in expired.into_iter() {
        remove_expired_assignment(guild, user, ctx).await?;
    }

    Ok(())
}

async fn remove_expired_assignment(guild: GuildId, user: UserId, ctx: &JobContext) -> Result<()> {
    // if we fail to remove the assignment, we can roll back the transaction
    // and try again later.
    let tx = ctx.db.begin().await?;

    let role = get_guild_role(guild, &ctx.db).await?;
    let mut member = guild.member(&ctx.ctx.http, user).await?;

    info!(
        "Removing role '{}' from member '{}' of guild '{}'",
        role,
        member.display_name(),
        guild.name(&ctx.ctx.cache).unwrap_or(guild.to_string()),
    );
    member.remove_role(&ctx.ctx.http, role).await?;

    info!("Role removed successfully, dropping DB record.");
    drop_expired_assignment(guild, user, &ctx.db).await?;

    tx.commit().await?;

    Ok(())
}

async fn get_expired_assignments(pool: &SqlitePool) -> Result<Vec<(GuildId, UserId)>> {
    let today = chrono::Utc::now();
    let rows: Vec<(String, String)> =
        sqlx::query_as(include_str!("queries/get_expired_meatball_assignments.sql"))
            .bind(today)
            .fetch_all(pool)
            .await?;

    let mut expired = vec![];
    for (guild, user) in rows.into_iter() {
        expired.push((GuildId(guild.parse()?), UserId(user.parse()?)));
    }

    Ok(expired)
}

async fn create_assignment(guild: GuildId, user: UserId, pool: &SqlitePool) -> Result<()> {
    let now = chrono::Utc::now();
    sqlx::query(include_str!("queries/create_meatball_assignment.sql"))
        .bind(guild.to_string())
        .bind(user.to_string())
        .bind(now)
        .execute(pool)
        .await?;
    Ok(())
}

async fn drop_expired_assignment(guild: GuildId, user: UserId, pool: &SqlitePool) -> Result<()> {
    sqlx::query(include_str!("queries/drop_meatball_assignment.sql"))
        .bind(guild.to_string())
        .bind(user.to_string())
        .execute(pool)
        .await?;
    Ok(())
}

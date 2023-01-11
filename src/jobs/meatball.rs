use color_eyre::Result;
use serenity::model::prelude::{ChannelId, GuildId, RoleId, UserId};
use sqlx::SqlitePool;
use tracing::info;

use crate::bot::JobContext;

pub(crate) async fn update_role_assignments(ctx: JobContext) -> Result<()> {
    let expired = expired_roles(&ctx.db).await?;

    for (guild, user) in expired.into_iter() {
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
        drop_expired_role(guild, user, &ctx.db).await?;
    }

    Ok(())
}

async fn get_guild_role(guild: GuildId, pool: &SqlitePool) -> Result<RoleId> {
    let result: (String,) = sqlx::query_as(include_str!("queries/get_guild_meatball_role.sql"))
        .bind(guild.to_string())
        .fetch_one(pool)
        .await?;

    Ok(RoleId(result.0.parse()?))
}

async fn get_guild_channel(guild: GuildId) -> ChannelId {
    ChannelId(0)
}

async fn expired_roles(pool: &SqlitePool) -> Result<Vec<(GuildId, UserId)>> {
    let today = chrono::Utc::now();
    let rows: Vec<(String, String)> =
        sqlx::query_as(include_str!("queries/expired_meatball_roles.sql"))
            .bind(today)
            .fetch_all(pool)
            .await?;

    let mut expired = vec![];
    for (guild, user) in rows.into_iter() {
        expired.push((GuildId(guild.parse()?), UserId(user.parse()?)));
    }

    Ok(expired)
}

async fn drop_expired_role(guild: GuildId, user: UserId, pool: &SqlitePool) -> Result<()> {
    sqlx::query(include_str!("queries/drop_expired_meatball_role.sql"))
        .bind(guild.to_string())
        .bind(user.to_string())
        .execute(pool)
        .await?;
    Ok(())
}

INSERT
INTO meatball_channel(
    guild_id,
    channel_id
)
VALUES(?, ?)
ON CONFLICT(guild_id) DO UPDATE SET
    channel_id = excluded.channel_id

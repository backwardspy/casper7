INSERT
INTO meatball_channels(
    guild_id,
    channel_id
)
VALUES(?, ?)
ON CONFLICT(guild_id) DO UPDATE SET
    channel_id = excluded.channel_id

INSERT
INTO meatball_roles(
    guild_id,
    role_id
)
VALUES(?, ?)
ON CONFLICT(guild_id) DO UPDATE SET
    role_id = excluded.role_id

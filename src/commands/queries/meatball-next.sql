SELECT
    user_id,
    day,
    month
FROM
    meatball_days
WHERE
    guild_id = ?

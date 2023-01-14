SELECT
    user_id,
    day,
    month
FROM
    meatball_day
WHERE
    guild_id = ?

SELECT
    user_id,
    month,
    day
FROM
    meatball_day
WHERE
    guild_id = ?

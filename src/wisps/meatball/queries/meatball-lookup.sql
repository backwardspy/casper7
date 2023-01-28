SELECT
    month,
    day
FROM
    meatball_day
WHERE
    guild_id = ? AND
    user_id = ?

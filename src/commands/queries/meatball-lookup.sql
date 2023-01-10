SELECT
    month,
    day
FROM
    meatball_days
WHERE
    guild_id = ? AND
    user_id = ?

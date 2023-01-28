SELECT
    md.guild_id,
    md.user_id
FROM
    meatball_day as md
LEFT JOIN
    meatball_role_assignment as mra
ON
    md.guild_id = mra.guild_id
    AND md.user_id = mra.user_id
WHERE
    mra.user_id IS NULL
    AND md.month = ?
    AND md.day = ?

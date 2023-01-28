DELETE
FROM
    meatball_role_assignment
WHERE
    guild_id = ? AND
    user_id = ?

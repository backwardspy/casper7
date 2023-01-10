INSERT INTO meatball_days (
       guild_id,
       user_id,
       month,
       day
) VALUES (?, ?, ?, ?)
ON CONFLICT(guild_id, user_id) DO UPDATE SET
   month = excluded.month,
   day = excluded.day

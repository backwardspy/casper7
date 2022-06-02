-- migrates the existing casper db to the new schema
BEGIN TRANSACTION;

-- meatball_days (old) -> meatball_day (new)
CREATE TABLE meatball_day (
    id INTEGER PRIMARY KEY,
    guild_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    month INTEGER NOT NULL,
    day INTEGER NOT NULL
);

CREATE UNIQUE INDEX idx_unique_guild_user_md ON meatball_day (guild_id, user_id);

INSERT INTO
    meatball_day (guild_id, user_id, month, day)
SELECT
    guild_id,
    user_id,
    month,
    day
FROM
    meatball_days;

DROP TABLE meatball_days;

-- meatball_channels (old) -> meatball_channel (new)
CREATE TABLE meatball_channel (
    id INTEGER PRIMARY KEY,
    guild_id TEXT NOT NULL,
    channel_id TEXT NOT NULL
);

CREATE UNIQUE INDEX idx_unique_guild_channel_mc ON meatball_channel (guild_id, channel_id);

INSERT INTO
    meatball_channel (guild_id, channel_id)
SELECT
    guild_id,
    channel_id
FROM
    meatball_channels;

DROP TABLE meatball_channels;

-- meatball_roles (old) -> meatball_role (new)
CREATE TABLE meatball_role (
    id INTEGER PRIMARY KEY,
    guild_id TEXT NOT NULL,
    role_id TEXT NOT NULL
);

CREATE UNIQUE INDEX idx_unique_guild_role_mr ON meatball_role (guild_id, role_id);

INSERT INTO
    meatball_role (guild_id, role_id)
SELECT
    guild_id,
    role_id
FROM
    meatball_roles;

DROP TABLE meatball_roles;

-- new table: meatball_role_assignment
CREATE TABLE meatball_role_assignment (
    id INTEGER PRIMARY KEY,
    guild_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    date DATE NOT NULL
);

CREATE UNIQUE INDEX idx_unique_guild_user_mra ON meatball_role_assignment (guild_id, user_id);

COMMIT;
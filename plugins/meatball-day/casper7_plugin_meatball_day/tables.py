"""Database table definitions."""

from piccolo.columns import Date, Integer, Text
from piccolo.table import Table


class MeatballDay(Table):
    """Stores meatball dates against a user & guild ID."""

    guild_id = Text()
    user_id = Text()
    month = Integer()
    day = Integer()


class MeatballChannel(Table):
    """Stores the channel to post in for each guild."""

    guild_id = Text()
    channel_id = Text()


class MeatballRole(Table):
    """Stores the role to assign on meatball day for each guild."""

    guild_id = Text()
    role_id = Text()


class MeatballRoleAssignment(Table):
    """Stores people who currently have the role assigned."""

    guild_id = Text()
    user_id = Text()
    date = Date()

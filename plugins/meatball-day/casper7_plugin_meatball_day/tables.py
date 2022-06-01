"""Database table definitions."""

from piccolo.columns import Integer, Text
from piccolo.table import Table


class MeatballDay(Table, tablename="meatball_days"):
    """Stores meatball dates against a user & guild ID."""

    guild_id = Text()
    user_id = Text()
    month = Integer()
    day = Integer()


class MeatballChannel(Table, tablename="meatball_channels"):
    """Stores the channel to post in for each guild."""

    guild_id = Text()
    channel_id = Text()


class MeatballRole(Table, tablename="meatball_roles"):
    """Stores the role to assign on meatball day for each guild."""

    guild_id = Text()
    role_id = Text()

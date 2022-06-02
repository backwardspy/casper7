"""
Meatball day casper7 plugin.

Usage:
    casper7-plugin-meatball-day [options] meatball-lookup [--] <args>
    casper7-plugin-meatball-day [options] meatball-save [--] <args>
    casper7-plugin-meatball-day [options] meatball-forget
    casper7-plugin-meatball-day [options] meatball-channel [--] <args>
    casper7-plugin-meatball-day [options] meatball-role [--] <args>
    casper7-plugin-meatball-day [options] meatball-next
    casper7-plugin-meatball-day refresh-role-assignments
    casper7-plugin-meatball-day --commands
    casper7-plugin-meatball-day --jobs
    casper7-plugin-meatball-day (-h | --help)
    casper7-plugin-meatball-day --version

Options:
    -g --guild GUILD-ID      Unique ID of the guild the command was sent from.
    -c --channel CHANNEL-ID  Unique ID of the channel the command was sent from.
    -u --user USER-ID        Unique ID of the user who sent the command.
    --commands               Print available commands in JSON format.
    --jobs                   Print jobs to be scheduled in JSON format.
    -h --help                Show this screen.
    --version                Show version.
"""
import asyncio
import itertools
import json
from importlib.metadata import version
from operator import itemgetter

import pendulum
import uvloop
from docopt import docopt

from casper7_plugin_meatball_day.db import upsert
from casper7_plugin_meatball_day.tables import (
    MeatballChannel,
    MeatballDay,
    MeatballRole,
    MeatballRoleAssignment,
)


def print_commands() -> None:
    """Print available commands in JSON format for casper7's query."""
    print(
        json.dumps(
            [
                {
                    "name": "meatball-lookup",
                    "description": "Find a user's meatball day.",
                    "args": [
                        {
                            "name": "user",
                            "description": "The user to lookup. Defaults to you.",
                            "type": "user",
                            "optional": True,
                        }
                    ],
                },
                {
                    "name": "meatball-next",
                    "description": "Find the next occurring meatball day.",
                },
                {
                    "name": "meatball-save",
                    "description": "Add your meatball day to the database.",
                    "args": [
                        {
                            "name": "day",
                            "description": "The day of your meatball day.",
                            "type": "int",
                        },
                        {
                            "name": "month",
                            "description": "The month of your meatball day.",
                            "type": "int",
                        },
                    ],
                },
                {
                    "name": "meatball-forget",
                    "description": "Remove your meatball day from the database.",
                },
                {
                    "name": "meatball-channel",
                    "description": "Set the channel to use for announcements.",
                    "admin": True,
                    "args": [
                        {
                            "name": "channel",
                            "description": "The channel to use for announcements.",
                            "type": "channel",
                        }
                    ],
                },
                {
                    "name": "meatball-role",
                    "description": "Set the role to assign on meatball day.",
                    "admin": True,
                    "args": [
                        {
                            "name": "role",
                            "description": "The role to assign on meatball day.",
                            "type": "role",
                        }
                    ],
                },
            ]
        )
    )


def print_jobs() -> None:
    """Prints scheduled jobs in JSON format for casper7's query."""
    print(
        json.dumps(
            [
                {
                    "name": "refresh-role-assignments",
                    "schedule": "0 10 * * *",
                }
            ]
        )
    )


async def meatball_lookup(user_id: str, *, guild_id: str) -> None:
    """Display a user's meatball day, if one is registered."""
    result = (
        await MeatballDay.select(MeatballDay.day, MeatballDay.month)
        .where(MeatballDay.guild_id == guild_id, MeatballDay.user_id == user_id)
        .first()
    )
    if result:
        day, month = itemgetter("day", "month")(result)
        print(
            f"<@{user_id}>'s meatball day is on {pendulum.date(1, month, day).format('MMMM Do')}! :mag:"
        )
    else:
        print(f"<@{user_id}> has not set their meatball day yet. :frowning:")


async def meatball_save(day: int, month: int, *, user_id: str, guild_id: str) -> None:
    """Saves a user's meatball day to the database."""
    if not 1 <= month <= 12:
        print(f"<@{user_id}> That's not a real month... :thinking:")
        return

    days = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31][month - 1]
    if not 1 <= day <= days:
        print(f"<@{user_id}> That's not a real day... :thinking:")
        return

    await upsert(
        MeatballDay,
        (MeatballDay.guild_id == guild_id) & (MeatballDay.user_id == user_id),
        {
            MeatballDay.month: month,
            MeatballDay.day: day,
        },
    )

    print(f"Registered <@{user_id}>'s meatball day! :calendar:")


async def meatball_forget(*, user_id: str, guild_id: str) -> None:
    """Remove a user's meatball day from the database."""
    await MeatballDay.delete().where(
        MeatballDay.guild_id == guild_id, MeatballDay.user_id == user_id
    )
    print(f"Removed <@{user_id}>'s meatball day from the database. :boom:")


async def meatball_channel(channel_id: str, *, guild_id: str) -> None:
    """Set the channel to use for announcements."""
    await upsert(
        MeatballChannel,
        MeatballChannel.guild_id == guild_id,
        {MeatballChannel.channel_id: channel_id},
    )
    print(f"Set the channel to use for announcements to <@{channel_id}>.")


async def meatball_role(role_id: str, *, guild_id: str) -> None:
    """Set the role to assign on meatball day."""
    await upsert(
        MeatballRole,
        MeatballRole.guild_id == guild_id,
        {MeatballRole.role_id: role_id},
    )
    print(f"Set the role to use for meatball day to <@{role_id}>.")


async def meatball_next(*, guild_id: str) -> None:
    """Display the next occurring meatball day."""
    today = pendulum.now().date()
    years = [today.year, today.add(years=1).year]

    # find meatball days for this guild
    results = await MeatballDay.select(
        MeatballDay.user_id, MeatballDay.day, MeatballDay.month
    ).where(MeatballDay.guild_id == guild_id)

    # convert into pendulum dates for this year and the next
    meatball_days = [
        (pendulum.date(year, result["month"], result["day"]), result["user_id"])
        for result, year in itertools.product(results, years)
    ]

    # sort by date in ascending order
    meatball_days.sort(key=itemgetter(0))

    # find the next one that's in the future
    date, user_id = next(
        meatball_day for meatball_day in meatball_days if meatball_day[0] > today
    )

    print(
        f"The next meatball day is <@{user_id}>'s on {date.format('MMMM Do, YYYY')}! :alarm_clock:"
    )


async def refresh_role_assignments() -> None:
    """
    Maintain the meatball role assignment table and tell casper7 to assign/remove roles.
    """
    events: list[dict] = []
    guild_roles: dict[str, str] = {}
    guild_channels: dict[str, str] = {}

    async def get_guild_role(guild_id: str) -> str:
        if role := guild_roles.get(guild_id):
            return role

        result = (
            await MeatballRole.select(MeatballRole.role_id)
            .where(MeatballRole.guild_id == guild_id)
            .first()
        )
        guild_roles[guild_id] = result["role_id"]
        return result["role_id"]

    async def get_guild_channel(guild_id: str) -> str:
        if role := guild_channels.get(guild_id):
            return role

        result = (
            await MeatballChannel.select(MeatballChannel.channel_id)
            .where(MeatballChannel.guild_id == guild_id)
            .first()
        )
        guild_channels[guild_id] = result["channel_id"]
        return result["channel_id"]

    today = pendulum.now().date()

    # assignments from days other than today should be removed.
    to_remove = await MeatballRoleAssignment.select(
        MeatballRoleAssignment.guild_id, MeatballRoleAssignment.user_id
    ).where(MeatballRoleAssignment.date != today.isoformat())

    for result in to_remove:
        guild_id, user_id = itemgetter("guild_id", "user_id")(result)
        await MeatballRoleAssignment.delete().where(
            MeatballRoleAssignment.guild_id == guild_id,
            MeatballRoleAssignment.user_id == user_id,
        )
        events.append(
            {
                "type": "remove_role",
                "guild_id": guild_id,
                "user_id": user_id,
                "role_id": await get_guild_role(guild_id),
            }
        )

    # meatball days for the current day should be added
    to_add = await MeatballDay.select(
        MeatballDay.guild_id,
        MeatballDay.user_id,
    ).where(MeatballDay.month == today.month, MeatballDay.day == today.day)

    for result in to_add:
        guild_id, user_id = itemgetter("guild_id", "user_id")(result)

        if await MeatballRoleAssignment.exists().where(
            MeatballRoleAssignment.guild_id == guild_id,
            MeatballRoleAssignment.user_id == user_id,
        ):
            continue

        guild_id, user_id = itemgetter("guild_id", "user_id")(result)
        await MeatballRoleAssignment(
            guild_id=guild_id,
            user_id=user_id,
            date=today.isoformat(),
        ).save()
        events.append(
            {
                "type": "add_role",
                "guild_id": guild_id,
                "user_id": user_id,
                "role_id": await get_guild_role(guild_id),
            }
        )
        events.append(
            {
                "type": "message",
                "channel_id": await get_guild_channel(guild_id),
                "text": f"It's <@{user_id}>'s meatball day! :partying_face::tada:",
            }
        )

    print(json.dumps(events))


async def _plugin() -> None:
    """Main entrypoint for the plugin."""
    args = docopt(
        __doc__,
        version=f"casper7-plugin-meatball-day {version(__package__)}",
        more_magic=True,
    )

    if args["<args>"]:
        args["<args>"] = json.loads(args["<args>"])

    guild_id = args["--guild"]
    channel_id = args["--channel"]
    user_id = args["--user"]

    match args:
        case {"--commands": True}:
            print_commands()
        case {"--jobs": True}:
            print_jobs()
        case {"meatball-lookup": True}:
            lookup_user = args["<args>"].get("user") or user_id
            await meatball_lookup(lookup_user, guild_id=guild_id)
        case {"meatball-save": True, "<args>": {"month": month, "day": day}}:
            await meatball_save(day, month, user_id=user_id, guild_id=guild_id)
        case {"meatball-forget": True}:
            await meatball_forget(user_id=user_id, guild_id=guild_id)
        case {"meatball-channel": True, "<args>": {"channel": channel_id}}:
            await meatball_channel(channel_id, guild_id=guild_id)
        case {"meatball-role": True, "<args>": {"role": role}}:
            await meatball_role(role, guild_id=guild_id)
        case {"meatball-next": True}:
            await meatball_next(guild_id=guild_id)
        case {"refresh-role-assignments": True}:
            await refresh_role_assignments()
        case _:
            raise ValueError(f"unknown argument set: {args}")


def plugin() -> None:
    """Entrypoint for the plugin, starts the async event loop."""
    uvloop.install()
    asyncio.run(_plugin())

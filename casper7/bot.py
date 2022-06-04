"""Casper7 pluggable discord bot."""
import json
import subprocess
from functools import cached_property
from operator import itemgetter
from pathlib import Path
from typing import Any

import lightbulb
import tomli
from apscheduler.schedulers.asyncio import AsyncIOScheduler
from apscheduler.triggers.cron import CronTrigger
from hikari import (
    InteractionChannel,
    InteractionMember,
    Member,
    OptionType,
    Permissions,
    Role,
    StartingEvent,
    User,
)
from pydantic import BaseModel, validator
from rich import print as rprint
from rich.console import Group
from rich.panel import Panel
from rich.table import Table

from casper7.settings import settings

ARG_TYPE_MAP = {
    "string": OptionType.STRING,
    "int": OptionType.INTEGER,
    "bool": OptionType.BOOLEAN,
    "user": OptionType.USER,
    "channel": OptionType.CHANNEL,
    "role": OptionType.ROLE,
    "mentionable": OptionType.MENTIONABLE,
    "float": OptionType.FLOAT,
}


def serialize_default(obj: Any) -> Any:
    """Serializes hikari types into orjson-compatible types."""
    match obj:
        case InteractionChannel():
            return obj.id
        case InteractionMember():
            return obj.id
        case Role():
            return obj.id
        case User():
            return obj.id
        case _:
            raise TypeError(f"Unable to serialize {repr(obj)}")


class PluginConfig(BaseModel):
    """Plugin configuration."""

    execute: str


class PluginCommandArgument(BaseModel):
    """An argument to a plugin command."""

    name: str
    description: str
    type: str = "string"
    optional: bool = False
    default: str | None = None

    @validator("type")
    @classmethod
    def validate_type(cls, v: str) -> str:
        """Validate type."""
        if v not in ARG_TYPE_MAP:
            raise ValueError(f"Invalid type: {v}. Must be one of {ARG_TYPE_MAP.keys()}")
        return v


class PluginCommand(BaseModel):
    """A command with 0 or more arguments. These are convert into slash commands."""

    name: str
    description: str
    admin: bool = False
    args: list[PluginCommandArgument] = []


class PluginJob(BaseModel):
    """A job that runs on a set schedule."""

    name: str
    schedule: str


class Plugin:
    """Wraps some sort of executable that provides slash commands."""

    def __init__(self, *, execute: str) -> None:
        self.execute = execute

    def issue_command(
        self, command: str, *, ctx: lightbulb.Context | None
    ) -> str | None:
        """Issue a command to the plugin."""
        args = [command]

        if ctx:
            if ctx.guild_id:
                args.extend(["--guild", str(ctx.guild_id)])

            args.extend(
                [
                    "--channel",
                    str(ctx.channel_id),
                    "--user",
                    str(ctx.author.id),
                ]
            )

            if ctx.raw_options:
                args.extend(
                    [
                        "--",
                        json.dumps(ctx.raw_options, default=serialize_default),
                    ]
                )

        try:
            proc = subprocess.run(
                self.execute.split() + args,
                stdout=subprocess.PIPE,
                check=True,
            )
        except subprocess.CalledProcessError as ex:
            print(f"Command {' '.join(args)} for plugin {self.version} failed: {ex}")
            raise

        return proc.stdout.decode("utf-8").strip()

    @cached_property
    def version(self) -> str:
        """
        Plugins should return version information in the form: NAME VERSION
        For example: casper7-plugin-example 1.0.0
        """
        result = self.issue_command("--version", ctx=None)

        if not result:
            print(f"Executable '{self.execute}' did not return version information")
            return "Unknown n/a"
        return result

    @cached_property
    def commands(self) -> list[PluginCommand]:
        """Query the plugin for its commands."""
        try:
            commands_json = self.issue_command("--commands", ctx=None)
        except subprocess.CalledProcessError as ex:
            print(f"Couldn't get commands for {self.version} ({ex})")
            return []

        if not commands_json:
            print(f"Plugin {self.version} did not return any commands.")
            return []

        return [PluginCommand(**command) for command in json.loads(commands_json)]

    @cached_property
    def jobs(self) -> list[PluginJob]:
        """Query the plugin for its jobs."""
        try:
            jobs_json = self.issue_command("--jobs", ctx=None)
        except subprocess.CalledProcessError as ex:
            print(f"Couldn't get jobs for {self.version} ({ex})")
            return []

        if not jobs_json:
            print(f"Plugin {self.version} did not return any jobs.")
            return []

        return [PluginJob(**job) for job in json.loads(jobs_json)]


async def _member_is_admin(member: Member) -> bool:
    return any(
        role.permissions & Permissions.ADMINISTRATOR
        for role in await member.fetch_roles()
    )


def _load_plugins() -> list[Plugin]:
    if not settings.plugins_file.exists():
        print(
            f"Plugins file {settings.plugins_file} does not exist, so no plugins will be loaded."
        )
        return []

    print(f"Loading plugins from {settings.plugins_file}")
    with settings.plugins_file.open() as plugins_file:
        return [Plugin(execute=line.strip()) for line in plugins_file]


def _register_commands(plugin: Plugin, *, lb_plugin: lightbulb.Plugin) -> None:
    for command in plugin.commands:

        @lightbulb.implements(lightbulb.SlashCommand)
        async def handler(
            ctx: lightbulb.Context,
            plugin: Plugin = plugin,
            command: PluginCommand = command,
        ) -> None:
            if command.admin and (
                ctx.member is None or not await _member_is_admin(ctx.member)
            ):
                await ctx.respond(
                    "**You must be an admin to use this command!** :police_officer:"
                )
                return

            response = plugin.issue_command(command.name, ctx=ctx)
            if not response:
                response = "*No response was returned, but the command succeeded.*"
            await ctx.respond(response)

        description = command.description
        if command.admin:
            description += " (Admin only!)"
        handler = lightbulb.command(command.name, description)(handler)

        for arg in command.args:
            handler = lightbulb.option(
                name=arg.name,
                description=arg.description,
                type=ARG_TYPE_MAP[arg.type],
                required=not arg.optional,
                default=arg.default,
            )(handler)

        lb_plugin.command(handler)


def _register_jobs(plugin: Plugin, *, bot: lightbulb.BotApp) -> None:
    for job in plugin.jobs:
        print(f"Registering job {job.name} on schedule {job.schedule}")

        async def handler(
            plugin: Plugin = plugin, job: PluginJob = job, bot: lightbulb.BotApp = bot
        ) -> None:
            print(f"Running job {job.name} for plugin {plugin.version}")
            events = json.loads(plugin.issue_command(job.name, ctx=None))
            rprint(events)

            for event in events:
                match event["type"]:
                    case "add_role":
                        guild_id, user_id, role_id = itemgetter(
                            "guild_id", "user_id", "role_id"
                        )(event)
                        await bot.rest.add_role_to_member(guild_id, user_id, role_id)
                    case "remove_role":
                        guild_id, user_id, role_id = itemgetter(
                            "guild_id", "user_id", "role_id"
                        )(event)
                        await bot.rest.remove_role_from_member(
                            guild_id, user_id, role_id
                        )
                    case "message":
                        channel_id, text = itemgetter("channel_id", "text")(event)
                        await bot.rest.create_message(channel_id, text)

        bot.d.scheduler.add_job(handler, CronTrigger.from_crontab(job.schedule))


def _register_plugins(plugins: list[Plugin], *, bot: lightbulb.BotApp) -> None:
    for plugin in plugins:
        name, version = plugin.version.split()
        lb_plugin = lightbulb.Plugin(name=name, description=f"{name} {version}")
        _register_commands(plugin, lb_plugin=lb_plugin)
        _register_jobs(plugin, bot=bot)
        bot.add_plugin(lb_plugin)


def _make_argument_table(command: PluginCommand) -> Table | str:
    if not command.args:
        return "[i]No arguments[/i]"

    table = Table(
        "Parameter",
        "Description",
        "Type",
        "Optional",
        style="magenta",
        header_style="magenta",
        expand=True,
    )
    for arg in command.args:
        table.add_row(
            arg.name, arg.description, arg.type, str(arg.optional), style="magenta"
        )
    return table


def _print_plugins(plugins: list[Plugin]) -> None:
    rprint(
        Group(
            *[
                Panel.fit(
                    Group(
                        *[
                            Panel(
                                _make_argument_table(command),
                                title=f"Command: {command.name}"
                                + (
                                    " ([b][red]admin[/red][/b])"
                                    if command.admin
                                    else ""
                                ),
                                style="yellow",
                            )
                            for command in plugin.commands
                        ],
                    ),
                    title=plugin.version,
                    subtitle=plugin.execute,
                    style="cyan",
                )
                for plugin in plugins
            ]
        )
    )


def make_bot() -> lightbulb.BotApp:
    """Sets up a bot instance."""
    bot = lightbulb.BotApp(
        settings.discord_token,
        default_enabled_guilds=[settings.testing_guild]
        if settings.testing_guild
        else (),
    )
    bot.d.scheduler = AsyncIOScheduler()

    @bot.listen(StartingEvent)
    async def _(_: StartingEvent) -> None:
        # at this point we should have an async event loop available.
        bot.d.scheduler.start()

    plugins = _load_plugins()

    if plugins:
        print("Loaded plugins:")
        _print_plugins(plugins)

        print("Registering plugins...")
        _register_plugins(plugins, bot=bot)

    return bot

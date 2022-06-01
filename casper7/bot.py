"""Casper7 pluggable discord bot."""
import json
import subprocess
from functools import cached_property
from pathlib import Path
from typing import Any

import lightbulb
import tomli
from hikari import (
    InteractionChannel,
    InteractionMember,
    Member,
    OptionType,
    Permissions,
    Role,
    User,
)
from pydantic import BaseModel, validator
from rich import print
from rich.console import Group
from rich.panel import Panel
from rich.table import Table

from settings import settings

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


class Plugin:
    """Wraps some sort of executable that provides slash commands."""

    def __init__(self, *, path: Path, execute: str) -> None:
        self.path = path
        self.execute = execute

    def issue_command(self, command: str, *, ctx: lightbulb.Context | None) -> str:
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

        proc = subprocess.run(
            self.execute.split() + args,
            cwd=self.path,
            stdout=subprocess.PIPE,
            check=True,
        )
        return proc.stdout.decode("utf-8").strip()

    @cached_property
    def version(self) -> str:
        """
        Plugins should return version information in the form: NAME VERSION
        For example: casper7-plugin-example 1.0.0
        """
        return self.issue_command("--version", ctx=None)

    @cached_property
    def commands(self) -> list[PluginCommand]:
        """Query the plugin for its commands."""
        return [
            PluginCommand(**command)
            for command in json.loads(self.issue_command("--commands", ctx=None))
        ]


async def _member_is_admin(member: Member) -> bool:
    return any(
        role.permissions & Permissions.ADMINISTRATOR
        for role in await member.fetch_roles()
    )


def _load_plugins() -> list[Plugin]:
    plugins = []
    for config_path in settings.plugins_root.glob("**/plugin.toml"):
        with config_path.open("rb") as config_file:
            config_data = tomli.load(config_file)
        config = PluginConfig(**config_data)
        plugins.append(Plugin(path=config_path.parent, execute=config.execute))

    return plugins


def _register_plugins(plugins: list[Plugin], *, bot: lightbulb.BotApp) -> None:
    for plugin in plugins:
        name, version = plugin.version.split()
        lb_plugin = lightbulb.Plugin(name=name, description=f"{name} {version}")

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
    print(
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
                    subtitle=str(plugin.path),
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
    plugins = _load_plugins()

    print("Loaded plugins:")
    _print_plugins(plugins)

    print("Registering plugin commands...")
    _register_plugins(plugins, bot=bot)

    return bot

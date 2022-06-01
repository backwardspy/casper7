"""Application settings module."""

from pathlib import Path

from pydantic import BaseSettings, Field


class Settings(BaseSettings):
    """Top level settings object."""

    # discord bot token from https://discordapp.com/developers/applications
    discord_token: str

    # if set, creates guild commands instead of global commands.
    testing_guild: int | None = None

    # where to look for plugins
    plugins_root: Path = Field("plugins")


settings = Settings()

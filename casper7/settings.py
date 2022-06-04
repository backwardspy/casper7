"""Application settings module."""

from pathlib import Path

import platformdirs
from pydantic import BaseSettings


class Settings(BaseSettings):
    """Top level settings object."""

    class Config:
        """Allows you to put your discord token into a docker secret."""
        secrets_dir = Path("/run/secrets")

    # the file to read plugin executables from, one per line.
    plugins_file: Path = platformdirs.user_config_path() / "plugins"

    # discord bot token from https://discordapp.com/developers/applications
    discord_token: str

    # if set, creates guild commands instead of global commands.
    testing_guild: int | None = None


settings = Settings()

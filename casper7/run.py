"""Casper7 scripts."""

from casper7.bot import make_bot


def bot() -> None:
    """Run the bot."""
    casper7 = make_bot()
    casper7.run()

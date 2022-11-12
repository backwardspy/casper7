"""Casper7 scripts."""

import sentry_sdk

from casper7.bot import make_bot
from casper7.settings import settings


def bot() -> None:
    """Run the bot."""
    if settings.sentry_dsn:
        sentry_sdk.init(
            dsn=settings.sentry_dsn,
            traces_sample_rate=1.0,
        )

    casper7 = make_bot()
    casper7.run()

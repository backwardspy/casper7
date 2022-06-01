"""
An example echo plugin that just says your message back to you.

Usage:
    casper7-plugin-echo echo <args>
    casper7-plugin-echo --commands
    casper7-plugin-echo (-h | --help)
    casper7-plugin-echo --version

Options:
    --commands    List available commands
    -h --help     Show this screen.
    --version     Show version.
"""
import json
from importlib.metadata import version
from typing import Any

from docopt import docopt


def list_commands() -> None:
    """Print the available commands."""
    print(
        json.dumps(
            [
                {
                    "name": "echo",
                    "description": "Read your message back to you.",
                    "args": [
                        {
                            "name": "message",
                            "description": "The message to read back to you.",
                        }
                    ],
                }
            ]
        )
    )


def echo(**kwargs: Any) -> None:
    """Read back the given message."""
    print(kwargs["message"])


def main() -> None:
    """Echo plugin CLI."""
    args = docopt(
        __doc__, version=f"casper7-plugin-echo {version('casper7-plugin-echo')}"
    )

    if args["--commands"]:
        list_commands()
    elif args["echo"]:
        echo(**json.loads(args["<args>"]))


if __name__ == "__main__":
    main()

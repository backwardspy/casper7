"""Database utility methods."""
from typing import Any, Type, TypeVar

from piccolo.columns import Column
from piccolo.custom_types import Combinable
from piccolo.table import Table

T = TypeVar("T", bound=Table)


async def upsert(
    table: Type[T], where: Combinable, defaults: dict[Column | str, Any]
) -> T:
    """Insert or update a row in the database."""
    record = await table.objects().get_or_create(where, defaults)
    if not record._was_created:  # pylint: disable=protected-access
        record = await table.update(defaults).where(where)  # noqa
    return record

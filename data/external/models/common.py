import pydantic
from pydantic.dataclasses import dataclass

PydanticConfiguration = pydantic.ConfigDict(
    frozen=True,
    str_strip_whitespace=True,
    extra=pydantic.Extra.forbid,
    populate_by_name=True,
    validate_default=True,
)


@dataclass(config=PydanticConfiguration)
class TranslatableStr:
    # pylint: disable-next=invalid-name
    de: str
    # pylint: disable-next=invalid-name
    en: str

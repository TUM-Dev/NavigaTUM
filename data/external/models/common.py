import pydantic
from pydantic.dataclasses import dataclass


# pylint: disable-next=too-few-public-methods
class PydanticConfiguration(pydantic.BaseConfig):
    allow_mutation = False
    frozen = True
    anystr_strip_whitespace = True
    extra = pydantic.Extra.forbid


@dataclass(config=PydanticConfiguration)
class TranslatableStr:
    # pylint: disable-next=invalid-name
    de: str
    # pylint: disable-next=invalid-name
    en: str

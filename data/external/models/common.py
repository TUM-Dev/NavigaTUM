from pathlib import Path

import pydantic
from pydantic import BaseModel


class PydanticConfiguration(BaseModel):
    model_config = pydantic.ConfigDict(
        frozen=False,
        str_strip_whitespace=True,
        extra="forbid",
        populate_by_name=True,
        validate_default=True,
    )


RESULTS = Path(__file__).parent.parent / "results"


class TranslatableStr(PydanticConfiguration):
    # pylint: disable-next=invalid-name
    de: str
    # pylint: disable-next=invalid-name
    en: str

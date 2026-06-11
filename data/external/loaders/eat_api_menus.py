import dataframely as dy
import polars as pl

from external.models.common import RESULTS_PATH
from external.schemas.eat_api_menus import EatApiMenuSchema

EAT_API_MENUS_CSV = RESULTS_PATH / "eat_api_menus.csv"


def load_eat_api_menus() -> dy.DataFrame[EatApiMenuSchema]:
    """Load and validate the cached `eat_api_menus.csv` weekly canteen menus."""
    df = pl.read_csv(EAT_API_MENUS_CSV, schema=EatApiMenuSchema.to_polars_schema())
    return EatApiMenuSchema.validate(df)

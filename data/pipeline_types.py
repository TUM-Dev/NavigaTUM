"""Named aliases for the pipeline's dynamic boundaries, so `disallow_any_explicit` bans bare `Any` everywhere else."""

from typing import Any

# Deserialized JSON/YAML value (source file, HTTP payload, (de)serialization helper).
type Json = Any  # type: ignore[explicit-any]
# A nested API location entry; recursive and field-optional across entry kinds, so no fixed TypedDict fits.
type Entry = dict[str, Json]
# A flat DataFrame row keyed by column name, as yielded by `iter_rows(named=True)` or built up by hand.
type FlatRow = dict[str, Json]

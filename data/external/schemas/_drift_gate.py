"""
Helper for drift-gate tests that surfaces row-level diagnostics on schema failure.

dataframely's default ``ValidationError`` reports failures as ``"<rule> failed for N rows"``
which is unactionable when an upstream sync introduces duplicate primary keys (a recurring
TUMonline drift mode). :func:`assert_satisfies_schema` lists the duplicated keys with their
occurrence counts so the failure can be patched or escalated without re-running the sync.
"""

import dataframely as dy
import polars as pl


def assert_satisfies_schema(schema: type[dy.Schema], df: pl.DataFrame) -> None:
    """Validate ``df`` against ``schema``, raising with row-level diagnostics on failure."""
    _, failure = schema.filter(df)
    if len(failure) == 0:
        return

    counts = failure.counts()
    pk_cols = schema.primary_key()
    sections: list[str] = []
    for rule, count in counts.items():
        if rule == "primary_key" and pk_cols:
            duplicates = (
                df.group_by(pk_cols)
                .agg(pl.len().alias("occurrences"))
                .filter(pl.col("occurrences") > 1)
                .sort("occurrences", *pk_cols, descending=[True, *([False] * len(pk_cols))])
            )
            with pl.Config(tbl_rows=50, fmt_str_lengths=80):
                sections.append(
                    f"- 'primary_key' failed for {count} row(s); "
                    f"{duplicates.height} duplicated key(s) on ({', '.join(pk_cols)}):\n"
                    f"{duplicates}"
                )
        else:
            sections.append(f"- {rule!r} failed for {count} row(s)")

    raise AssertionError(f"{schema.__name__} validation failed:\n" + "\n".join(sections))

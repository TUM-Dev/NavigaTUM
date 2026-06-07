import dataframely as dy
import polars as pl

# RFC 3339 / ISO 8601 with mandatory timezone offset.
# Datetimes are persisted as strings so the Rust parquet reader can parse them
# with chrono::DateTime::parse_from_rfc3339; this rule keeps that contract.
_ISO8601_TZ_REGEX = r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})$"

# Locally-hosted CDN delivery path, never an external URL. Event images live in the shared repo
# image tree (`data/sources/img/lg/<id>_<index>.webp`, source/license in `img-sources.yaml`), are
# resized into `thumb/` by the image pipeline, and ship under `/cdn`. The stored value is the
# server-relative path the photo marker fetches verbatim as `${cdnURL}<value>` — so the storage
# layout, not the frontend, owns where the image lives. Pinned to `thumb/` (the 256x256 square the
# marker renders) and to the `<id>_<index>.webp` shape of `parse_image_filename`. The leading
# `/cdn/` keeps `http(s)://…` external hosts out by construction.
_LOCAL_IMAGE_REGEX = r"^/cdn/thumb/[A-Za-z0-9.\-]+_[0-9]+\.webp$"


class EventsSchema(dy.Schema):
    """Schema for the campus-events catalogue (`events.parquet`)."""

    image = dy.String(nullable=False)
    lat = dy.Float64(nullable=False, min=-90.0, max=90.0)
    lon = dy.Float64(nullable=False, min=-180.0, max=180.0)
    name = dy.String(nullable=False)
    starts_at = dy.String(nullable=False)
    ends_at = dy.String(nullable=False)
    description = dy.String(nullable=False)
    organising_org_id = dy.Int32(nullable=False)

    @dy.rule()
    def name_non_empty(cls) -> pl.Expr:
        """`name` must be a non-empty string after trimming."""
        return pl.col("name").str.strip_chars().str.len_chars() > 0

    @dy.rule()
    def image_is_local_cdn_path(cls) -> pl.Expr:
        """`image` must be a local `/cdn/thumb/…` delivery path, not an external URL."""
        return pl.col("image").str.contains(_LOCAL_IMAGE_REGEX)

    @dy.rule()
    def starts_at_is_rfc3339(cls) -> pl.Expr:
        """`starts_at` must be an RFC 3339 timestamp with a timezone offset."""
        return pl.col("starts_at").str.contains(_ISO8601_TZ_REGEX)

    @dy.rule()
    def ends_at_is_rfc3339(cls) -> pl.Expr:
        """`ends_at` must be an RFC 3339 timestamp with a timezone offset."""
        return pl.col("ends_at").str.contains(_ISO8601_TZ_REGEX)

    @dy.rule()
    def ends_at_not_before_starts_at(cls) -> pl.Expr:
        """`ends_at` must be lexicographically >= `starts_at` (matches DB CHECK)."""
        return pl.col("ends_at") >= pl.col("starts_at")

    @dy.rule()
    def organising_org_id_positive(cls) -> pl.Expr:
        """`organising_org_id` must be a positive TUMonline org_id."""
        return pl.col("organising_org_id") > 0

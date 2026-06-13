import dataframely as dy
import polars as pl

# RFC 3339 / ISO 8601 with mandatory timezone offset.
# Datetimes are persisted as strings so the Rust parquet reader can parse them
# with chrono::DateTime::parse_from_rfc3339; this rule keeps that contract.
_ISO8601_TZ_REGEX = r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})$"

# Locally-hosted CDN delivery path the photo marker fetches as `${cdnURL}<value>`, not an external URL.
# The base name may contain underscores (webform submissions are named `event_<hash>`).
_LOCAL_IMAGE_REGEX = r"^/cdn/thumb/[A-Za-z0-9._\-]+_[0-9]+\.webp$"


class EventsSchema(dy.Schema):
    """Schema for the campus-events catalogue (`events.parquet`)."""

    image = dy.String(nullable=False)
    lat = dy.Float64(nullable=False, min=-90.0, max=90.0)
    lon = dy.Float64(nullable=False, min=-180.0, max=180.0)
    name = dy.String(nullable=False)
    starts_at = dy.String(nullable=False)
    ends_at = dy.String(nullable=False)
    # Derived server-side visibility gate (processors.events_appears_at); never reaches the client.
    appears_at = dy.String(nullable=False)
    description = dy.String(nullable=False)
    organising_org_id = dy.Int32(nullable=False)
    image_author = dy.String(nullable=False)

    @dy.rule()
    def name_non_empty(cls) -> pl.Expr:
        """`name` must be a non-empty string after trimming."""
        return pl.col("name").str.strip_chars().str.len_chars() > 0

    @dy.rule()
    def image_author_non_empty(cls) -> pl.Expr:
        """`image_author` must be a non-empty string after trimming - CC-BY requires attribution."""
        return pl.col("image_author").str.strip_chars().str.len_chars() > 0

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
    def appears_at_is_rfc3339(cls) -> pl.Expr:
        """`appears_at` must be an RFC 3339 timestamp with a timezone offset."""
        return pl.col("appears_at").str.contains(_ISO8601_TZ_REGEX)

    @dy.rule()
    def ends_at_not_before_starts_at(cls) -> pl.Expr:
        """`ends_at` must be lexicographically >= `starts_at` (matches DB CHECK)."""
        return pl.col("ends_at") >= pl.col("starts_at")

    @dy.rule()
    def organising_org_id_positive(cls) -> pl.Expr:
        """`organising_org_id` must be a positive TUMonline org_id."""
        return pl.col("organising_org_id") > 0

import dataframely as dy
import polars as pl
from external.schemas.events import EventsSchema

from processors.export import event_search_documents


def _events(rows: list[dict[str, object]]) -> dy.DataFrame[EventsSchema]:
    """Build a validated events frame from row dicts, defaulting every required column."""
    defaults: dict[str, object] = {
        "image": "/cdn/thumb/event_9d02ddd940c43f87_0.webp",
        "lat": 48.262908,
        "lon": 11.669102,
        "name": "GARNIX Festival",
        "starts_at": "2026-06-15T16:00:00+02:00",
        "ends_at": "2026-06-19T23:59:00+02:00",
        "description": "Open-air student festival.",
        "organising_org_id": 51897,
        "image_author": "Studentische Vertretung TUM",
    }
    df = pl.DataFrame([defaults | row for row in rows], schema=EventsSchema.to_polars_schema())
    return EventsSchema.validate(df)


def test_each_row_becomes_one_event_document() -> None:
    """One search document per CSV row, faceted as `event`, with the key-plus-date `ms_id`."""
    docs = event_search_documents(_events([{}]))

    assert len(docs) == 1
    doc = docs[0]
    assert doc["facet"] == "event"
    # The `event_<hash>` identity from the image path, plus the UTC start date for
    # uniqueness against legacy duplicate keys.
    assert doc["ms_id"] == "event_9d02ddd940c43f87_2026-06-15"
    assert doc["name"] == "GARNIX Festival"


def test_datetimes_are_normalised_to_utc() -> None:
    """Offsets are converted to `Z` so Meilisearch's lexicographic sort on `starts_at` is chronological."""
    docs = event_search_documents(
        _events(
            [
                {"starts_at": "2026-06-15T16:00:00+02:00", "ends_at": "2026-06-19T23:59:00+02:00"},
                {"starts_at": "2026-06-29T09:00:00.000Z", "ends_at": "2026-07-03T21:00:00.000Z"},
            ],
        ),
    )

    assert docs[0]["starts_at"] == "2026-06-15T14:00:00Z"
    assert docs[0]["ends_at"] == "2026-06-19T21:59:00Z"
    assert docs[1]["starts_at"] == "2026-06-29T09:00:00Z"
    assert docs[1]["ends_at"] == "2026-07-03T21:00:00Z"


def test_document_carries_the_full_prefill_payload() -> None:
    """Everything a client needs to pre-fill the event proposal form rides along on the document."""
    doc = event_search_documents(_events([{}]))[0]

    assert doc["key"] == "event_9d02ddd940c43f87"
    assert doc["description"] == "Open-air student festival."
    assert doc["organising_org_id"] == 51897
    assert doc["image"] == "/cdn/thumb/event_9d02ddd940c43f87_0.webp"
    assert doc["image_author"] == "Studentische Vertretung TUM"
    assert doc["_geo"] == {"lat": 48.262908, "lng": 11.669102}
    # Lecture precedent: a uniform `rank` of 0 keeps the `rank:desc` ranking rule neutral.
    assert doc["rank"] == 0


def test_duplicate_keys_across_editions_get_distinct_ms_ids() -> None:
    """Legacy rows may reuse one key for several editions; the date suffix keeps `ms_id` unique."""
    docs = event_search_documents(
        _events(
            [
                {"starts_at": "2025-06-16T16:00:00+02:00", "ends_at": "2025-06-20T23:59:00+02:00"},
                {"starts_at": "2026-06-15T16:00:00+02:00", "ends_at": "2026-06-19T23:59:00+02:00"},
            ],
        ),
    )

    assert docs[0]["key"] == docs[1]["key"]
    assert docs[0]["ms_id"] != docs[1]["ms_id"]

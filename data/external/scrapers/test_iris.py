import logging

import pytest
import requests

from external.scrapers import iris


class _FakeResponse:
    def __init__(self, payload: dict):
        self._payload = payload

    def raise_for_status(self) -> None:
        pass

    def json(self) -> dict:
        return self._payload


def test_fetch_returns_parsed_rooms_on_success(monkeypatch):
    """A well-formed Iris response is parsed into IrisRoom models."""
    payload = {"raeume": [{"raum_nr_architekt": "D 11@4113", "gebaeude_code": "4113"}]}
    monkeypatch.setattr(requests, "get", lambda *a, **kw: _FakeResponse(payload))

    rooms = iris.fetch_iris_rooms()

    assert rooms == [iris.IrisRoom(raum_nr_architekt="D 11@4113", gebaeude_code="4113")]


def test_fetch_returns_none_and_warns_when_iris_unreachable(monkeypatch, caplog):
    """A transient AStA outage must degrade silently: return None, log a warning, never raise."""

    def _boom(*args, **kwargs):
        raise requests.ConnectionError("iris is down")

    monkeypatch.setattr(requests, "get", _boom)

    with caplog.at_level(logging.WARNING):
        rooms = iris.fetch_iris_rooms()

    assert rooms is None
    assert caplog.records, "expected a warning to be logged"


def test_fetch_returns_none_on_malformed_payload(monkeypatch):
    """A response missing the expected `raeume` shape must not raise."""
    monkeypatch.setattr(requests, "get", lambda *a, **kw: _FakeResponse({"unexpected": True}))

    assert iris.fetch_iris_rooms() is None

import base64
import itertools
import json
import zlib
from uuid import uuid4

import requests

from lib.api_config import BASE_URL
from lib.tracing import Phases


def test_get_room_events_returns_200_with_the_board_as_the_first_event(
    phases: Phases,
):
    phases.arrange()
    text = uuid4().hex
    dsl = f'# eventstorming v1\n\nevent     e1   "{text}" @ 0,0\n'
    compressor = zlib.compressobj(wbits=-15)
    data = (
        base64.urlsafe_b64encode(compressor.compress(dsl.encode()) + compressor.flush())
        .decode()
        .rstrip("=")
    )
    created = requests.post(f"{BASE_URL}/rooms", json={"data": data})
    assert created.status_code == 201

    phases.act()
    with requests.get(
        f"{BASE_URL}/rooms/{created.json()['id']}/events", stream=True, timeout=5
    ) as response:
        lines = response.iter_lines(decode_unicode=True)
        event = dict(line.split(": ", 1) for line in itertools.takewhile(bool, lines))

    phases.assert_()
    assert response.status_code == 200
    assert response.headers["Content-Type"] == "text/event-stream"
    assert event["id"] == "0"
    assert event["event"] == "snapshot"
    assert json.loads(event["data"]) == {"seq": 0, "dsl": dsl}


def test_get_room_events_after_ops_returns_the_board_with_them_applied(
    phases: Phases,
):
    phases.arrange()
    dsl = '# eventstorming v1\n\nevent     e1   "" @ 0,0\n'
    compressor = zlib.compressobj(wbits=-15)
    data = (
        base64.urlsafe_b64encode(compressor.compress(dsl.encode()) + compressor.flush())
        .decode()
        .rstrip("=")
    )
    created = requests.post(f"{BASE_URL}/rooms", json={"data": data})
    assert created.status_code == 201
    posted = requests.post(
        f"{BASE_URL}/rooms/{created.json()['id']}/ops",
        json={
            "ops": [
                {"type": "moveNotes", "positions": [{"id": "e1", "x": 16, "y": 32}]}
            ]
        },
    )
    assert posted.status_code == 200

    phases.act()
    with requests.get(
        f"{BASE_URL}/rooms/{created.json()['id']}/events", stream=True, timeout=5
    ) as response:
        lines = response.iter_lines(decode_unicode=True)
        event = dict(line.split(": ", 1) for line in itertools.takewhile(bool, lines))

    phases.assert_()
    assert response.status_code == 200
    assert event["id"] == "1"
    assert event["event"] == "snapshot"
    assert json.loads(event["data"]) == {
        "seq": 1,
        "dsl": '# eventstorming v1\n\nevent     e1   "" @ 16,32\n',
    }


def test_get_room_events_then_post_room_ops_sends_the_applied_ops_as_an_event(
    phases: Phases,
):
    phases.arrange()
    dsl = '# eventstorming v1\n\nevent     e1   "" @ 0,0\n'
    compressor = zlib.compressobj(wbits=-15)
    data = (
        base64.urlsafe_b64encode(compressor.compress(dsl.encode()) + compressor.flush())
        .decode()
        .rstrip("=")
    )
    created = requests.post(f"{BASE_URL}/rooms", json={"data": data})
    assert created.status_code == 201
    text = uuid4().hex

    with requests.get(
        f"{BASE_URL}/rooms/{created.json()['id']}/events", stream=True, timeout=5
    ) as response:
        lines = response.iter_lines(decode_unicode=True)
        snapshot = dict(
            line.split(": ", 1) for line in itertools.takewhile(bool, lines)
        )
        assert snapshot["event"] == "snapshot"

        phases.act()
        posted = requests.post(
            f"{BASE_URL}/rooms/{created.json()['id']}/ops",
            json={
                "ops": [
                    {
                        "type": "addNote",
                        "noteType": "event",
                        "id": "e1",
                        "text": text,
                        "x": 16,
                        "y": 32,
                    },
                    {
                        "type": "moveNotes",
                        "positions": [{"id": f"e{uuid4().hex}", "x": 0, "y": 0}],
                    },
                    {"type": "moveNotes", "positions": [{"id": "e1", "x": 8, "y": 8}]},
                ]
            },
        )
        assert posted.status_code == 200
        event = dict(line.split(": ", 1) for line in itertools.takewhile(bool, lines))

    phases.assert_()
    assert event["id"] == "1"
    assert event["event"] == "edited"
    assert json.loads(event["data"]) == {
        "seq": 1,
        "edits": [
            {
                "type": "noteAdded",
                "noteType": "event",
                "id": "e2",
                "text": text,
                "x": 16,
                "y": 32,
            },
            {"type": "notesMoved", "positions": [{"id": "e1", "x": 8, "y": 8}]},
        ],
    }


def test_get_room_events_does_not_send_ops_of_other_room(phases: Phases):
    phases.arrange()
    dsl = '# eventstorming v1\n\nevent     e1   "" @ 0,0\n'
    compressor = zlib.compressobj(wbits=-15)
    data = (
        base64.urlsafe_b64encode(compressor.compress(dsl.encode()) + compressor.flush())
        .decode()
        .rstrip("=")
    )
    created = requests.post(f"{BASE_URL}/rooms", json={"data": data})
    assert created.status_code == 201
    other = requests.post(f"{BASE_URL}/rooms", json={"data": data})
    assert other.status_code == 201
    ops = {
        "ops": [{"type": "moveNotes", "positions": [{"id": "e1", "x": 16, "y": 32}]}]
    }

    with requests.get(
        f"{BASE_URL}/rooms/{created.json()['id']}/events", stream=True, timeout=5
    ) as response:
        lines = response.iter_lines(decode_unicode=True)
        snapshot = dict(
            line.split(": ", 1) for line in itertools.takewhile(bool, lines)
        )
        assert snapshot["event"] == "snapshot"

        phases.act()
        for room in [other, other, created]:
            posted = requests.post(
                f"{BASE_URL}/rooms/{room.json()['id']}/ops", json=ops
            )
            assert posted.status_code == 200
        event = dict(line.split(": ", 1) for line in itertools.takewhile(bool, lines))

    phases.assert_()
    assert event["id"] == "1"
    assert event["event"] == "edited"


def test_get_room_events_with_id_of_no_room_returns_404(phases: Phases):
    phases.act()
    response = requests.get(f"{BASE_URL}/rooms/{uuid4()}/events", timeout=5)

    phases.assert_()
    assert response.status_code == 404
    assert response.headers["Content-Type"] == "application/problem+json"
    assert response.json() == {
        "type": "about:blank",
        "title": "Not Found",
        "status": 404,
    }


def test_get_room_events_with_id_not_uuid_returns_400(phases: Phases):
    phases.act()
    response = requests.get(f"{BASE_URL}/rooms/{uuid4().hex[:8]}/events", timeout=5)

    phases.assert_()
    assert response.status_code == 400

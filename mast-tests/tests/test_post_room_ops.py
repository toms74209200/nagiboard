import base64
import zlib
from uuid import uuid4

import requests

from lib.api_config import BASE_URL
from lib.tracing import Phases


def test_post_room_ops_with_add_note_of_free_id_returns_200_with_it_applied(
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

    phases.act()
    response = requests.post(
        f"{BASE_URL}/rooms/{created.json()['id']}/ops",
        json={
            "ops": [
                {
                    "type": "addNote",
                    "noteType": "command",
                    "id": f"c{uuid4().hex}",
                    "text": uuid4().hex,
                    "x": 16,
                    "y": 32,
                }
            ]
        },
    )

    phases.assert_()
    assert response.status_code == 200
    assert response.json() == {"seq": 1, "results": [{"status": "applied"}]}


def test_post_room_ops_with_add_note_of_taken_id_returns_200_with_it_reassigned(
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

    phases.act()
    response = requests.post(
        f"{BASE_URL}/rooms/{created.json()['id']}/ops",
        json={
            "ops": [
                {
                    "type": "addNote",
                    "noteType": "event",
                    "id": "e1",
                    "text": uuid4().hex,
                    "x": 16,
                    "y": 32,
                }
            ]
        },
    )

    phases.assert_()
    assert response.status_code == 200
    assert response.json() == {
        "seq": 1,
        "results": [{"status": "reassigned", "id": "e2"}],
    }


def test_post_room_ops_with_move_notes_of_missing_note_returns_200_with_it_skipped(
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

    phases.act()
    response = requests.post(
        f"{BASE_URL}/rooms/{created.json()['id']}/ops",
        json={
            "ops": [
                {
                    "type": "moveNotes",
                    "positions": [
                        {"id": "e1", "x": 16, "y": 32},
                        {"id": f"e{uuid4().hex}", "x": 0, "y": 0},
                    ],
                }
            ]
        },
    )

    phases.assert_()
    assert response.status_code == 200
    assert response.json() == {"seq": 0, "results": [{"status": "skipped"}]}


def test_post_room_ops_with_operation_on_note_added_in_the_same_batch_returns_200_with_it_applied(
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
    id = f"c{uuid4().hex}"

    phases.act()
    response = requests.post(
        f"{BASE_URL}/rooms/{created.json()['id']}/ops",
        json={
            "ops": [
                {
                    "type": "addNote",
                    "noteType": "command",
                    "id": id,
                    "text": "",
                    "x": 0,
                    "y": 0,
                },
                {"type": "moveNotes", "positions": [{"id": id, "x": 16, "y": 32}]},
            ]
        },
    )

    phases.assert_()
    assert response.status_code == 200
    assert response.json() == {
        "seq": 1,
        "results": [{"status": "applied"}, {"status": "applied"}],
    }


def test_post_room_ops_twice_returns_the_next_seq_for_the_second_batch(
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
    ops = {
        "ops": [{"type": "moveNotes", "positions": [{"id": "e1", "x": 16, "y": 32}]}]
    }
    first = requests.post(f"{BASE_URL}/rooms/{created.json()['id']}/ops", json=ops)
    assert first.status_code == 200

    phases.act()
    response = requests.post(f"{BASE_URL}/rooms/{created.json()['id']}/ops", json=ops)

    phases.assert_()
    assert response.status_code == 200
    assert response.json()["seq"] == first.json()["seq"] + 1


def test_post_room_ops_with_id_of_no_room_returns_404(phases: Phases):
    phases.act()
    response = requests.post(
        f"{BASE_URL}/rooms/{uuid4()}/ops",
        json={
            "ops": [{"type": "moveNotes", "positions": [{"id": "e1", "x": 0, "y": 0}]}]
        },
    )

    phases.assert_()
    assert response.status_code == 404
    assert response.headers["Content-Type"] == "application/problem+json"
    assert response.json() == {
        "type": "about:blank",
        "title": "Not Found",
        "status": 404,
    }


def test_post_room_ops_with_id_not_uuid_returns_400(phases: Phases):
    phases.act()
    response = requests.post(
        f"{BASE_URL}/rooms/{uuid4().hex[:8]}/ops",
        json={
            "ops": [{"type": "moveNotes", "positions": [{"id": "e1", "x": 0, "y": 0}]}]
        },
    )

    phases.assert_()
    assert response.status_code == 400


def test_post_room_ops_without_operations_returns_400(phases: Phases):
    phases.act()
    response = requests.post(f"{BASE_URL}/rooms/{uuid4()}/ops", json={"ops": []})

    phases.assert_()
    assert response.status_code == 400


def test_post_room_ops_with_unknown_note_type_returns_400(phases: Phases):
    phases.act()
    response = requests.post(
        f"{BASE_URL}/rooms/{uuid4()}/ops",
        json={
            "ops": [
                {
                    "type": "addNote",
                    "noteType": uuid4().hex,
                    "id": "e1",
                    "text": "",
                    "x": 0,
                    "y": 0,
                }
            ]
        },
    )

    phases.assert_()
    assert response.status_code == 400

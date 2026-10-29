import base64
import zlib
from uuid import uuid4

import requests

from lib.api_config import BASE_URL


def test_get_board_with_id_of_board_of_created_room_returns_200_with_its_content():
    dsl = f'# eventstorming v1\n\nevent     e1   "{uuid4().hex}" @ 0,0\n'
    compressor = zlib.compressobj(wbits=-15)
    data = (
        base64.urlsafe_b64encode(compressor.compress(dsl.encode()) + compressor.flush())
        .decode()
        .rstrip("=")
    )
    created = requests.post(f"{BASE_URL}/rooms", json={"data": data})
    assert created.status_code == 201
    boards = requests.get(f"{BASE_URL}/rooms/{created.json()['id']}/boards")
    assert boards.status_code == 200

    response = requests.get(f"{BASE_URL}/boards/{boards.json()['boards'][0]}")

    assert response.status_code == 200
    assert response.headers["Content-Type"] == "text/plain"
    assert response.text == dsl


def test_get_board_with_id_of_board_of_created_room_by_another_user_returns_the_same_content():
    dsl = f'# eventstorming v1\n\nevent     e1   "{uuid4().hex}" @ 0,0\n'
    compressor = zlib.compressobj(wbits=-15)
    data = (
        base64.urlsafe_b64encode(compressor.compress(dsl.encode()) + compressor.flush())
        .decode()
        .rstrip("=")
    )
    created = requests.post(f"{BASE_URL}/rooms", json={"data": data})
    assert created.status_code == 201
    another_user = requests.Session()
    boards = another_user.get(f"{BASE_URL}/rooms/{created.json()['id']}/boards")
    assert boards.status_code == 200

    response = another_user.get(f"{BASE_URL}/boards/{boards.json()['boards'][0]}")

    assert response.status_code == 200
    assert response.text == dsl


def test_get_board_with_id_of_board_of_room_with_applied_operations_returns_its_content_with_them():
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
    applied = requests.post(
        f"{BASE_URL}/rooms/{created.json()['id']}/ops",
        json={
            "ops": [
                {
                    "type": "addNote",
                    "noteType": "command",
                    "id": "c1",
                    "text": text,
                    "x": 216,
                    "y": 240,
                },
                {"type": "moveNotes", "positions": [{"id": "e1", "x": 16, "y": 32}]},
            ]
        },
    )
    assert applied.status_code == 200
    boards = requests.get(f"{BASE_URL}/rooms/{created.json()['id']}/boards")
    assert boards.status_code == 200

    response = requests.get(f"{BASE_URL}/boards/{boards.json()['boards'][0]}")

    assert response.status_code == 200
    assert response.text == (
        "# eventstorming v1\n\n"
        'event     e1   "" @ 16,32\n'
        f'command   c1   "{text}" @ 216,240\n'
    )


def test_get_board_with_id_of_no_board_returns_404():
    response = requests.get(f"{BASE_URL}/boards/{uuid4()}")

    assert response.status_code == 404
    assert response.headers["Content-Type"] == "application/problem+json"
    assert response.json() == {
        "type": "about:blank",
        "title": "Not Found",
        "status": 404,
    }


def test_get_board_with_id_not_uuid_returns_400():
    response = requests.get(f"{BASE_URL}/boards/{uuid4().hex[:8]}")

    assert response.status_code == 400

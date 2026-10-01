import base64
import re
import zlib
from uuid import uuid4

import requests

from lib.api_config import BASE_URL

UUID4_PATTERN = re.compile(
    r"^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$"
)


def test_get_room_boards_with_id_of_created_room_returns_200_with_its_board():
    dsl = f'# eventstorming v1\n\nevent     e1   "{uuid4().hex}" @ 0,0\n'
    compressor = zlib.compressobj(wbits=-15)
    data = (
        base64.urlsafe_b64encode(compressor.compress(dsl.encode()) + compressor.flush())
        .decode()
        .rstrip("=")
    )
    created = requests.post(f"{BASE_URL}/rooms", json={"data": data})
    assert created.status_code == 201

    response = requests.get(f"{BASE_URL}/rooms/{created.json()['id']}/boards")

    assert response.status_code == 200
    boards = response.json()["boards"]
    assert len(boards) == 1
    assert UUID4_PATTERN.match(boards[0])


def test_get_room_boards_with_id_of_no_room_returns_404():
    response = requests.get(f"{BASE_URL}/rooms/{uuid4()}/boards")

    assert response.status_code == 404
    assert response.headers["Content-Type"] == "application/problem+json"
    assert response.json() == {
        "type": "about:blank",
        "title": "Not Found",
        "status": 404,
    }


def test_get_room_boards_with_id_not_uuid_returns_400():
    response = requests.get(f"{BASE_URL}/rooms/{uuid4().hex[:8]}/boards")

    assert response.status_code == 400

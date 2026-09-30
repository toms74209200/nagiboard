import base64
import random
import re
import string
import zlib
from uuid import uuid4

import pytest
import requests

from lib.api_config import BASE_URL

UUID4_PATTERN = re.compile(
    r"^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$"
)


def test_post_rooms_with_data_of_board_returns_201():
    dsl = f'# eventstorming v1\n\nevent e{uuid4().hex[:8]} "{uuid4().hex}" @ 0,0\n'
    compressor = zlib.compressobj(wbits=-15)
    data = (
        base64.urlsafe_b64encode(compressor.compress(dsl.encode()) + compressor.flush())
        .decode()
        .rstrip("=")
    )

    response = requests.post(f"{BASE_URL}/rooms", json={"data": data})

    assert response.status_code == 201
    assert UUID4_PATTERN.match(response.json()["id"])


def test_post_rooms_twice_returns_different_ids():
    dsl = f'# eventstorming v1\n\nevent e{uuid4().hex[:8]} "{uuid4().hex}" @ 0,0\n'
    compressor = zlib.compressobj(wbits=-15)
    data = (
        base64.urlsafe_b64encode(compressor.compress(dsl.encode()) + compressor.flush())
        .decode()
        .rstrip("=")
    )
    first = requests.post(f"{BASE_URL}/rooms", json={"data": data})
    assert first.status_code == 201

    response = requests.post(f"{BASE_URL}/rooms", json={"data": data})

    assert response.status_code == 201
    assert response.json()["id"] != first.json()["id"]


def test_post_rooms_with_data_not_raw_deflate_returns_422():
    data = base64.urlsafe_b64encode(b"\x07" + uuid4().bytes).decode().rstrip("=")

    response = requests.post(f"{BASE_URL}/rooms", json={"data": data})

    assert response.status_code == 422
    assert response.headers["Content-Type"] == "application/problem+json"
    assert response.json() == {
        "type": "about:blank",
        "title": "Unprocessable Content",
        "status": 422,
    }


def test_post_rooms_with_dsl_exceeding_1mib_returns_422():
    dsl = f'event e{uuid4().hex[:8]} "" @ 0,0' + "\n" * (1 << 20)
    compressor = zlib.compressobj(wbits=-15)
    data = (
        base64.urlsafe_b64encode(compressor.compress(dsl.encode()) + compressor.flush())
        .decode()
        .rstrip("=")
    )

    response = requests.post(f"{BASE_URL}/rooms", json={"data": data})

    assert response.status_code == 422
    assert response.headers["Content-Type"] == "application/problem+json"
    assert "diagnostics" not in response.json()


def test_post_rooms_with_unknown_note_type_returns_422_with_its_diagnostic():
    note_type = "".join(random.choices(string.ascii_lowercase, k=12))
    dsl = f'# eventstorming v1\n\n{note_type} x{uuid4().hex[:8]} "" @ 0,0\n'
    compressor = zlib.compressobj(wbits=-15)
    data = (
        base64.urlsafe_b64encode(compressor.compress(dsl.encode()) + compressor.flush())
        .decode()
        .rstrip("=")
    )

    response = requests.post(f"{BASE_URL}/rooms", json={"data": data})

    assert response.status_code == 422
    assert response.headers["Content-Type"] == "application/problem+json"
    diagnostics = response.json()["diagnostics"]
    assert len(diagnostics) == 1
    assert diagnostics[0]["line"] == 3
    assert note_type in diagnostics[0]["message"]


@pytest.mark.parametrize("character", ["+", "/", "="])
def test_post_rooms_with_data_outside_base64url_returns_400(character):
    data = (
        "".join(random.choices(string.ascii_letters + string.digits + "-_", k=16))
        + character
    )

    response = requests.post(f"{BASE_URL}/rooms", json={"data": data})

    assert response.status_code == 400


def test_post_rooms_with_data_exceeding_max_length_returns_400():
    data = "".join(random.choices(string.ascii_letters + string.digits + "-_", k=16385))

    response = requests.post(f"{BASE_URL}/rooms", json={"data": data})

    assert response.status_code == 400


def test_post_rooms_with_data_of_max_length_returns_422():
    response = requests.post(f"{BASE_URL}/rooms", json={"data": "A" * 16384})

    assert response.status_code == 422
    assert response.headers["Content-Type"] == "application/problem+json"


@pytest.mark.parametrize("body", [{}, {"data": 1}, {"data": None}, {"data": ["A"]}])
def test_post_rooms_with_body_not_matching_schema_returns_422(body):
    response = requests.post(f"{BASE_URL}/rooms", json=body)

    assert response.status_code == 422


@pytest.mark.parametrize("body", ["{", ""])
def test_post_rooms_with_body_not_json_returns_400(body):
    response = requests.post(
        f"{BASE_URL}/rooms",
        data=body,
        headers={"Content-Type": "application/json"},
    )

    assert response.status_code == 400


def test_post_rooms_without_json_content_type_returns_415():
    response = requests.post(
        f"{BASE_URL}/rooms",
        data=f'{{"data": "{uuid4().hex}"}}',
        headers={"Content-Type": "text/plain"},
    )

    assert response.status_code == 415

import base64
import random
import re
import string
import zlib
from uuid import uuid4

import pytest
import requests

from lib.api_config import BASE_URL
from lib.tracing import Phases

UUID4_PATTERN = re.compile(
    r"^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$"
)


def test_post_rooms_with_data_of_board_returns_201(phases: Phases):
    phases.arrange()
    dsl = f'# eventstorming v1\n\nevent e{uuid4().hex[:8]} "{uuid4().hex}" @ 0,0\n'
    compressor = zlib.compressobj(wbits=-15)
    data = (
        base64.urlsafe_b64encode(compressor.compress(dsl.encode()) + compressor.flush())
        .decode()
        .rstrip("=")
    )

    phases.act()
    response = requests.post(f"{BASE_URL}/rooms", json={"data": data})

    phases.assert_()
    assert response.status_code == 201
    assert UUID4_PATTERN.match(response.json()["id"])
    assert response.json()["shareUrl"] == f"/?room={response.json()['id']}"


def test_post_rooms_with_data_already_posted_returns_different_id(phases: Phases):
    phases.arrange()
    dsl = f'# eventstorming v1\n\nevent e{uuid4().hex[:8]} "{uuid4().hex}" @ 0,0\n'
    compressor = zlib.compressobj(wbits=-15)
    data = (
        base64.urlsafe_b64encode(compressor.compress(dsl.encode()) + compressor.flush())
        .decode()
        .rstrip("=")
    )
    first = requests.post(f"{BASE_URL}/rooms", json={"data": data})
    assert first.status_code == 201

    phases.act()
    response = requests.post(f"{BASE_URL}/rooms", json={"data": data})

    phases.assert_()
    assert response.status_code == 201
    assert response.json()["id"] != first.json()["id"]


def test_post_rooms_with_data_not_raw_deflate_returns_400(phases: Phases):
    phases.arrange()
    data = base64.urlsafe_b64encode(b"\x07" + uuid4().bytes).decode().rstrip("=")

    phases.act()
    response = requests.post(f"{BASE_URL}/rooms", json={"data": data})

    phases.assert_()
    assert response.status_code == 400
    assert response.headers["Content-Type"] == "application/problem+json"
    assert response.json() == {
        "type": "about:blank",
        "title": "Bad Request",
        "status": 400,
    }


def test_post_rooms_with_dsl_exceeding_1mib_returns_400(phases: Phases):
    phases.arrange()
    dsl = f'event e{uuid4().hex[:8]} "" @ 0,0' + "\n" * (1 << 20)
    compressor = zlib.compressobj(wbits=-15)
    data = (
        base64.urlsafe_b64encode(compressor.compress(dsl.encode()) + compressor.flush())
        .decode()
        .rstrip("=")
    )

    phases.act()
    response = requests.post(f"{BASE_URL}/rooms", json={"data": data})

    phases.assert_()
    assert response.status_code == 400
    assert response.headers["Content-Type"] == "application/problem+json"
    assert "diagnostics" not in response.json()


def test_post_rooms_with_unknown_note_type_returns_400_with_its_diagnostic(
    phases: Phases,
):
    phases.arrange()
    note_type = "".join(random.choices(string.ascii_lowercase, k=12))
    dsl = f'# eventstorming v1\n\n{note_type} x{uuid4().hex[:8]} "" @ 0,0\n'
    compressor = zlib.compressobj(wbits=-15)
    data = (
        base64.urlsafe_b64encode(compressor.compress(dsl.encode()) + compressor.flush())
        .decode()
        .rstrip("=")
    )

    phases.act()
    response = requests.post(f"{BASE_URL}/rooms", json={"data": data})

    phases.assert_()
    assert response.status_code == 400
    assert response.headers["Content-Type"] == "application/problem+json"
    diagnostics = response.json()["diagnostics"]
    assert len(diagnostics) == 1
    assert diagnostics[0]["line"] == 3
    assert note_type in diagnostics[0]["message"]


@pytest.mark.parametrize("character", ["+", "/", "="])
def test_post_rooms_with_data_outside_base64url_returns_400(character, phases: Phases):
    phases.arrange()
    data = (
        "".join(random.choices(string.ascii_letters + string.digits + "-_", k=16))
        + character
    )

    phases.act()
    response = requests.post(f"{BASE_URL}/rooms", json={"data": data})

    phases.assert_()
    assert response.status_code == 400


def test_post_rooms_with_data_exceeding_max_length_returns_400(phases: Phases):
    phases.arrange()
    data = "".join(random.choices(string.ascii_letters + string.digits + "-_", k=16385))

    phases.act()
    response = requests.post(f"{BASE_URL}/rooms", json={"data": data})

    phases.assert_()
    assert response.status_code == 400


def test_post_rooms_with_data_of_max_length_returns_400_as_problem(phases: Phases):
    phases.act()
    response = requests.post(f"{BASE_URL}/rooms", json={"data": "A" * 16384})

    phases.assert_()
    assert response.status_code == 400
    assert response.headers["Content-Type"] == "application/problem+json"


@pytest.mark.parametrize("body", [{}, {"data": 1}, {"data": None}, {"data": ["A"]}])
def test_post_rooms_with_body_not_matching_schema_returns_422(body, phases: Phases):
    phases.act()
    response = requests.post(f"{BASE_URL}/rooms", json=body)

    phases.assert_()
    assert response.status_code == 422


@pytest.mark.parametrize("body", ["{", ""])
def test_post_rooms_with_body_not_json_returns_400(body, phases: Phases):
    phases.act()
    response = requests.post(
        f"{BASE_URL}/rooms",
        data=body,
        headers={"Content-Type": "application/json"},
    )

    phases.assert_()
    assert response.status_code == 400


def test_post_rooms_without_json_content_type_returns_415(phases: Phases):
    phases.act()
    response = requests.post(
        f"{BASE_URL}/rooms",
        data=f'{{"data": "{uuid4().hex}"}}',
        headers={"Content-Type": "text/plain"},
    )

    phases.assert_()
    assert response.status_code == 415

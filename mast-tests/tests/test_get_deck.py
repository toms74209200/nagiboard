import base64
import zlib
from urllib.parse import urljoin
from uuid import uuid4

import requests

from lib.api_config import BASE_URL
from lib.tracing import Phases


def test_get_deck_with_share_url_of_created_room_returns_200_with_app_html(
    phases: Phases,
):
    phases.arrange()
    dsl = f'# eventstorming v1\n\nevent e{uuid4().hex[:8]} "{uuid4().hex}" @ 0,0\n'
    compressor = zlib.compressobj(wbits=-15)
    data = (
        base64.urlsafe_b64encode(compressor.compress(dsl.encode()) + compressor.flush())
        .decode()
        .rstrip("=")
    )
    created = requests.post(f"{BASE_URL}/rooms", json={"data": data})
    assert created.status_code == 201

    phases.act()
    response = requests.get(urljoin(BASE_URL, created.json()["shareUrl"]))

    phases.assert_()
    assert response.status_code == 200
    assert response.headers["Content-Type"] == "text/html"
    assert response.text.lower().startswith("<!doctype html>")


def test_get_deck_with_room_not_uuid_returns_400(phases: Phases):
    phases.act()
    response = requests.get(f"{BASE_URL}/", params={"room": uuid4().hex[:8]})

    phases.assert_()
    assert response.status_code == 400


def test_get_deck_with_post_method_returns_405(phases: Phases):
    phases.act()
    response = requests.post(f"{BASE_URL}/", data=uuid4().hex)

    phases.assert_()
    assert response.status_code == 405

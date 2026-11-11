from uuid import uuid4

import requests

from lib.api_config import BASE_URL
from lib.tracing import Phases


def test_get_llms_txt_returns_200_with_how_to_join_a_room(phases: Phases):
    phases.act()
    response = requests.get(f"{BASE_URL}/llms.txt")

    phases.assert_()
    assert response.status_code == 200
    assert response.headers["Content-Type"] == "text/markdown"
    assert response.text.startswith("# nagiboard\n")
    assert "/rooms/<room-id>/boards" in response.text
    assert "/rooms/<room-id>/ops" in response.text
    assert "/rooms/<room-id>/events" in response.text


def test_get_llms_txt_with_post_method_returns_405(phases: Phases):
    phases.act()
    response = requests.post(f"{BASE_URL}/llms.txt", data=uuid4().hex)

    phases.assert_()
    assert response.status_code == 405

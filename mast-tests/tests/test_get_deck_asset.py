import re
from uuid import uuid4

import requests

from lib.api_config import BASE_URL


def test_get_deck_asset_with_name_the_app_page_refers_to_returns_200_as_javascript():
    page = requests.get(f"{BASE_URL}/")
    (script,) = re.findall(r'src="/assets/([^"]+)"', page.text)

    response = requests.get(f"{BASE_URL}/assets/{script}")

    assert response.status_code == 200
    assert response.headers["Content-Type"] == "text/javascript"


def test_get_deck_asset_with_name_of_no_file_returns_404():
    response = requests.get(f"{BASE_URL}/assets/{uuid4().hex}.js")

    assert response.status_code == 404
    assert response.headers["Content-Type"] == "application/problem+json"
    assert response.json() == {
        "type": "about:blank",
        "title": "Not Found",
        "status": 404,
    }


def test_get_deck_asset_with_name_outside_the_pattern_returns_400():
    for name in [f"{uuid4().hex}.css", f"..{uuid4().hex}.js", "%2E%2E%2Findex.html"]:
        response = requests.get(f"{BASE_URL}/assets/{name}")

        assert response.status_code == 400, name

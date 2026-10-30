import re
from uuid import uuid4

import requests

from lib.api_config import BASE_URL
from lib.tracing import Phases


def test_get_deck_asset_with_name_the_app_page_refers_to_returns_200_as_javascript(
    phases: Phases,
):
    phases.arrange()
    page = requests.get(f"{BASE_URL}/")
    (script,) = re.findall(r'src="/assets/([^"]+)"', page.text)

    phases.act()
    response = requests.get(f"{BASE_URL}/assets/{script}")

    phases.assert_()
    assert response.status_code == 200
    assert response.headers["Content-Type"] == "text/javascript"


def test_get_deck_asset_with_name_of_no_file_returns_404(phases: Phases):
    phases.act()
    response = requests.get(f"{BASE_URL}/assets/{uuid4().hex}.js")

    phases.assert_()
    assert response.status_code == 404
    assert response.headers["Content-Type"] == "application/problem+json"
    assert response.json() == {
        "type": "about:blank",
        "title": "Not Found",
        "status": 404,
    }


def test_get_deck_asset_with_name_outside_the_pattern_returns_400(phases: Phases):
    for name in [f"{uuid4().hex}.css", f"..{uuid4().hex}.js", "%2E%2E%2Findex.html"]:
        phases.act()
        response = requests.get(f"{BASE_URL}/assets/{name}")

        phases.assert_()
        assert response.status_code == 400, name

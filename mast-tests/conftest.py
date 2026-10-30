from collections.abc import Generator
from pathlib import Path

import pytest

from lib.tracing import Phases, TestTiming, get_collector, render_table

_SPAN_REPORT_OPTION = "--span-report"
_REPORT_FILE = Path("reports/spanTiming/spanTiming.txt")


def pytest_addoption(parser: pytest.Parser) -> None:
    parser.addoption(_SPAN_REPORT_OPTION, action="store_true", default=False)


@pytest.fixture
def phases(request: pytest.FixtureRequest) -> Generator[Phases]:
    enabled: bool = request.config.getoption(_SPAN_REPORT_OPTION)
    p = Phases(enabled=enabled)
    yield p
    p.finish()
    if enabled:
        phase_nanos = p.get_phase_nanos()
        get_collector().record(
            TestTiming(
                test_module=request.node.module.__name__,
                test_name=request.node.name,
                arrange_nanos=phase_nanos["arrange"],
                act_nanos=phase_nanos["act"],
                assert_nanos=phase_nanos["assert"],
            )
        )


def pytest_sessionfinish(session: pytest.Session, exitstatus: int) -> None:
    try:
        span_report: bool = session.config.getoption(_SPAN_REPORT_OPTION)
    except ValueError:
        return
    if not span_report:
        return
    timings = get_collector().all()
    if not timings:
        return
    table = render_table(timings)
    print(f"\n{table}")
    _REPORT_FILE.parent.mkdir(parents=True, exist_ok=True)
    _REPORT_FILE.write_text(table)

import threading
from dataclasses import dataclass

from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import SimpleSpanProcessor
from opentelemetry.sdk.trace.export.in_memory_span_exporter import InMemorySpanExporter
from opentelemetry.trace import Span, Tracer

_TEST_HEADER = "Test"
_ARRANGE_HEADER = "Arrange (ms)"
_ACT_HEADER = "Act (ms)"
_ASSERT_HEADER = "Assert (ms)"
_TOTAL_HEADER = "Total (ms)"
_MAX_TEST_COLUMN_WIDTH = 60


@dataclass
class TestTiming:
    test_module: str
    test_name: str
    arrange_nanos: int
    act_nanos: int
    assert_nanos: int


class TestTimingCollector:
    def __init__(self) -> None:
        self._timings: list[TestTiming] = []
        self._lock = threading.Lock()

    def record(self, timing: TestTiming) -> None:
        with self._lock:
            self._timings.append(timing)

    def all(self) -> list[TestTiming]:
        with self._lock:
            return list(self._timings)


_collector = TestTimingCollector()


def get_collector() -> TestTimingCollector:
    return _collector


class Phases:
    def __init__(self, enabled: bool = True) -> None:
        self._enabled = enabled
        self._exporter: InMemorySpanExporter | None = None
        self._tracer: Tracer | None = None
        self._current_span: Span | None = None

        if enabled:
            self._exporter = InMemorySpanExporter()
            provider = TracerProvider()
            provider.add_span_processor(SimpleSpanProcessor(self._exporter))
            self._tracer = provider.get_tracer("api-test")

    def arrange(self) -> None:
        self._switch_to("arrange")

    def act(self) -> None:
        self._switch_to("act")

    def assert_(self) -> None:
        self._switch_to("assert")

    def _switch_to(self, name: str) -> None:
        if not self._enabled or self._tracer is None:
            return
        if self._current_span is not None:
            self._current_span.end()
        self._current_span = self._tracer.start_span(name)

    def finish(self) -> None:
        if self._current_span is not None:
            self._current_span.end()
            self._current_span = None

    def get_phase_nanos(self) -> dict[str, int]:
        result: dict[str, int] = {"arrange": 0, "act": 0, "assert": 0}
        if self._exporter is None:
            return result
        for span in self._exporter.get_finished_spans():
            if span.name in result:
                result[span.name] += span.end_time - span.start_time
        return result


def render_table(timings: list[TestTiming]) -> str:
    def fmt(value: float) -> str:
        return f"{value:.2f}"

    rows = sorted(
        [
            {
                "test": f"{t.test_module} > {t.test_name}",
                "arrange_ms": t.arrange_nanos / 1_000_000.0,
                "act_ms": t.act_nanos / 1_000_000.0,
                "assert_ms": t.assert_nanos / 1_000_000.0,
                "total_ms": (t.arrange_nanos + t.act_nanos + t.assert_nanos)
                / 1_000_000.0,
            }
            for t in timings
        ],
        key=lambda r: r["total_ms"],
        reverse=True,
    )

    test_width = min(
        _MAX_TEST_COLUMN_WIDTH,
        max(len(_TEST_HEADER), max(len(r["test"]) for r in rows)),
    )
    arrange_width = max(
        len(_ARRANGE_HEADER), max(len(fmt(r["arrange_ms"])) for r in rows)
    )
    act_width = max(len(_ACT_HEADER), max(len(fmt(r["act_ms"])) for r in rows))
    assert_width = max(len(_ASSERT_HEADER), max(len(fmt(r["assert_ms"])) for r in rows))
    total_width = max(len(_TOTAL_HEADER), max(len(fmt(r["total_ms"])) for r in rows))

    def make_row(
        test_lines: list[str], arrange: str, act: str, assert_val: str, total: str
    ) -> str:
        result_lines = []
        for i, line in enumerate(test_lines):
            arr = arrange if i == 0 else ""
            a = act if i == 0 else ""
            asr = assert_val if i == 0 else ""
            tot = total if i == 0 else ""
            result_lines.append(
                f"{line:<{test_width}} | {arr:>{arrange_width}} | {a:>{act_width}}"
                f" | {asr:>{assert_width}} | {tot:>{total_width}}"
            )
        return "\n".join(result_lines)

    def wrap_test_name(name: str) -> list[str]:
        if len(name) <= test_width:
            return [name]
        lines: list[str] = []
        remaining = name
        while len(remaining) > test_width:
            break_at = remaining.rfind(" ", 0, test_width)
            if break_at <= 0:
                break_at = test_width
            lines.append(remaining[:break_at].rstrip())
            remaining = remaining[break_at:].lstrip()
        lines.append(remaining)
        return lines

    header = make_row(
        [_TEST_HEADER], _ARRANGE_HEADER, _ACT_HEADER, _ASSERT_HEADER, _TOTAL_HEADER
    )
    separator = "".join("+" if c == "|" else "-" for c in header.split("\n")[0])
    body_rows = [
        make_row(
            wrap_test_name(r["test"]),
            fmt(r["arrange_ms"]),
            fmt(r["act_ms"]),
            fmt(r["assert_ms"]),
            fmt(r["total_ms"]),
        )
        for r in rows
    ]

    return "\n".join([header, separator] + body_rows)

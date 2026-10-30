# mast API Tests

## Environments

- Python
- [uv](https://docs.astral.sh/uv/)

## Setup

```bash
uv sync
```

## Usage

Run the server (`mast/`) first, then:

```bash
uv run pytest tests/ -vv
```

To report the time each test spends in its arrange, act, and assert phases (also written to
`reports/spanTiming/spanTiming.txt`):

```bash
uv run pytest tests/ -vv --span-report
```

## Environment Variables

- `BASE_URL`: API server base URL (default: `http://localhost:8080`)

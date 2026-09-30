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

## Environment Variables

- `BASE_URL`: API server base URL (default: `http://localhost:8080`)

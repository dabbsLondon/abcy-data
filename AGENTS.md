# Agent Instructions

This repository exposes a small HTTP API for working with cycling data. The agent can use these endpoints from any language. Examples below use Python.

## Starting the server

Run the server in development using Cargo:

```bash
cargo run
```

The server listens on `localhost:8080` by default.

## HTTP endpoints

- `GET /activities` – returns a JSON array of activity IDs that have been processed.
- `GET /raw` – returns a JSON array of `.fit` file names in `DATA_DIR`.

## Python usage

The `requests` library is sufficient for interacting with the API:

```python
import requests

base = "http://localhost:8080"

activities = requests.get(f"{base}/activities").json()
raw_files = requests.get(f"{base}/raw").json()
print(activities, raw_files)
```

The service automatically downloads the last ten Strava activities on startup and checks for new ones every five minutes.

The included `abcy-data.postman_collection.json` can be imported into Postman for manual exploration of the API.

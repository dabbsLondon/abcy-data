# Agent Instructions

This repository exposes a small HTTP API for working with cycling data. The agent can use these endpoints from any language. Examples below use Python.

## Starting the server

Run the server in development using Cargo:

```bash
cargo run
```

The server listens on `localhost:8080` by default.

## HTTP endpoints

- `GET /activities` – list downloaded activities (id, name, date, distance)
- `GET /activity/{id}` – full metadata and streams for an activity
- `GET /files` – recursive listing of stored files
- `POST /webhook` – Strava webhook used to trigger immediate downloads

## Python usage

The `requests` library is sufficient for interacting with the API:

```python
import requests

base = "http://localhost:8080"

activities = requests.get(f"{base}/activities").json()
if activities:
    first = requests.get(f"{base}/activity/{activities[0]['id']}").json()
else:
    first = None
print(activities, first is not None)
```

The service downloads the most recent Strava activities on startup. Use the `/webhook` endpoint to fetch new data as Strava notifies the server.

The included `abcy-data.postman_collection.json` can be imported into Postman for manual exploration of the API.

## Modules

- **auth** – handles Strava OAuth and token refresh logic
- **fetch** – downloads activity metadata and streams
- **storage** – writes and reads compressed JSON files on disk
- **web** – exposes the HTTP API routes
- **schema** – shared structs for metadata and stream payloads
- **utils** – helpers and application configuration

Downloaded activities are stored under `DATA_DIR/<user>/<year>/<id>` where
`<user>` is configured in `config.toml`. Each directory contains
`meta.json.zst` and `streams.json.zst`.

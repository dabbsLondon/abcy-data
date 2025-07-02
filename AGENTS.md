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
- `GET /activity/{id}` – full metadata and streams (time and power) for an activity
- `GET /files` – recursive listing of stored files
- `GET /ftp` – return the current FTP value
- `GET /ftp/history?count=n` – return FTP history ordered by newest first, optionally limited to `n` entries
- `POST /ftp` – append a new FTP value
- `GET /weight` – return the current weight in kilograms
- `GET /weight/history?count=n` – return weight history ordered by newest first, optionally limited to `n` entries
- `POST /weight` – append a new weight value
- `GET /wkg` – return watts per kilogram using the current FTP and weight
- `GET /wkg/history?count=n` – return stored watts per kilogram history
- `GET /enduro` – compute the current EnduroScore and store it
- `GET /enduro/history?count=n` – return EnduroScore history
- `GET /fitness` – compute the current FitnessScore and store it
- `GET /fitness/history?count=n` – return FitnessScore history
- `GET /openapi.json` – machine-readable OpenAPI description of all endpoints
- `POST /webhook` – Strava webhook used to trigger immediate downloads
- `GET /stats?period=week&ids=1,2&types=Ride` – aggregated statistics grouped by day, week,
  month or year. Optional comma-separated `ids` and `types` filters restrict the
  activities considered. The `period` parameter accepts `day`, `week`, `month`
  or `year`.
  The response includes ride count, total distance, average weighted power,
  average intensity factor, total training stress and average speed for each
  period returned.

Activity summaries now include normalized power (NP), intensity factor (IF) and training stress score (TSS) calculated using the stored FTP value.

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
- **fetch** – downloads activity metadata and streams (time and power)
- **storage** – writes and reads compressed JSON files on disk
- **web** – exposes the HTTP API routes
- **schema** – shared structs for metadata and stream payloads
- **utils** – helpers and application configuration

Downloaded activities are stored under `DATA_DIR/<user>/<year>/<id>` where
`<user>` is configured in `config.toml`. Each directory contains
`meta.json.zst` and `streams.json.zst`.
The `<user>` directory also stores `ftp.json`, `weight.json` and
`wkg.json` tracking your FTP, weight and watts-per-kilogram history, and
`enduro.json` and `fitness.json` storing the ride readiness scores.

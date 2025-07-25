# abcy-data

This project synchronizes cycling activities from Strava and exposes a simple HTTP API.  Activity metadata and data streams—including power output—are stored as compressed JSON files for further analysis.

## Requirements

- **Rust** (edition 2021)
- A Strava account with API credentials

## Configuration

Configuration is read from `config.toml` instead of environment variables. A minimal example is provided in `config.example.toml`. Copy this file to `config.toml` and fill in your credentials:

```toml
[strava]
client_id = "12345"
client_secret = "secret"
token_path = "./token.json"      # cached access token and expiry

[storage]
data_dir = "./data"
download_count = 10
user = "athlete1"
```

If the cached token has expired or the Strava API returns a `401`, the
application launches the OAuth flow again so you can re-authorize access.
The new access token and expiry time are written to `token_path` and the expiry
is logged.

### Obtaining Strava Tokens

1. Create an application at <https://www.strava.com/settings/api> to receive a client ID and secret.
2. Authorize the application for your account by visiting a URL such as:

   ```text
   https://www.strava.com/oauth/authorize?client_id=<client id>&response_type=code&redirect_uri=http://localhost/exchange_token&approval_prompt=force&scope=activity:read_all
   ```

   After approving you will be redirected with a `code` query parameter.
3. Instead of manually exchanging the code you can run the included
   `authorize` binary which performs the full OAuth flow automatically. The
   binary reads `config.toml` for the Strava client credentials and writes the
   returned tokens to the configured `token_path`:

   ```bash
   cargo run --bin authorize
   ```

   This opens a browser window, waits for the redirect and saves the returned
   credentials to the path specified by `token_path`.

4. If you prefer the manual approach, exchange the code via curl:

   ```bash
   curl -X POST https://www.strava.com/oauth/token \
       -d client_id=<client id> \
       -d client_secret=<client secret> \
       -d code=<authorization code> \
       -d grant_type=authorization_code
   ```

  The JSON response includes a `refresh_token`, but this application ignores it
  and will prompt for authorization again when needed.

## Running

```bash
cargo run
```

The `default-run` target is configured, so this command starts the main
`abcy-data` server without needing `--bin`.

On startup the app will:

1. Query your most recent activities (count configured by `download_count`).
2. Fetch metadata and data streams for each new activity (time and power when
   available) and store them under `DATA_DIR/<user>/<year>/<id>/` as
   `meta.json.zst` and `streams.json.zst`. During this step the service
   computes normalized power (NP), intensity factor (IF) and training stress
   score (TSS) using the current FTP value and writes them into `meta.json.zst`.
3. Start an HTTP server on `localhost:8080`.

### Ride Readiness Scores

The service tracks two additional metrics derived from your recent training:

- **EnduroScore** – gauges long-ride durability using the average distance and
  duration of your long rides, weekly volume and the last four weeks of training
  stress. The score decays if no long ride was completed in the past 14 days.
- **FitnessScore** – reflects overall aerobic conditioning. It combines weekly
  training hours (times four), the four‑week average Training Stress Score
  (divided by ten) and a bonus for frequent long rides. After three consecutive
  rest days the score decreases by 1.5% per day.

Scores roughly range as follows:

- 80–100: Event-ready endurance and fitness
- 60–79: Solid aerobic base
- 40–59: Building phase
- < 40: Detraining or early base period

### API Endpoints

- `GET /activities?count=n` – list activities ordered by newest first. If `count` is omitted all headers are returned.
- `GET /activity/{id}` – full metadata and streams (time and power) for an activity.
- `GET /activity/{id}/summary` – small summary including duration, weighted average power, average speed, intensity factor, training stress score and average heart rate. The response includes a `trend` section comparing recent rides.
- `GET /files` – recursive listing of everything under `DATA_DIR`.
- `GET /raw/{path}` – return a stored file by relative path.
- `GET /ftp` – return the current FTP value.
- `GET /ftp/history?count=n` – return the stored FTP history ordered by newest first, optionally limited to `n` items.
- `POST /ftp` – append a new FTP value.
- `GET /weight` – return the current weight in kilograms.
- `GET /weight/history?count=n` – return weight history ordered by newest first, optionally limited to `n` items.
- `POST /weight` – append a new weight value.
- `GET /wkg` – return the current watts per kilogram using FTP and weight.
- `GET /wkg/history?count=n` – return stored watts per kilogram history.
- `GET /enduro` – compute the current EnduroScore and store it.
- `GET /enduro/history?count=n` – return EnduroScore history ordered by newest first.
- `GET /fitness` – compute the current FitnessScore and store it.
- `GET /fitness/history?count=n` – return FitnessScore history ordered by newest first.
- `GET /trend` – return performance trends comparing the last three months of rides to the prior three months. This is the same data available in the `trend` field of activity summaries.
- `GET /openapi.json` – machine-readable OpenAPI description of all endpoints.
- `GET /stats?period=week&ids=1,2&types=Ride` – aggregated statistics grouped by day, week,
  month or year. Optional filters allow specifying a comma-separated list of activity
  IDs with `ids` and a list of activity types with `types` (e.g. `Ride`, `Run`).
  Available IDs can be obtained from the `/activities` endpoint.
- `POST /webhook` – Strava webhook endpoint used to fetch new data immediately.

A Postman collection `abcy-data.postman_collection.json` is included to help
test the endpoints. Set the `base_url` variable to your server's address.

Example statistics request:

```bash
curl "http://localhost:8080/stats?period=month&ids=123,124&types=Ride"
```

This returns aggregated metrics for the selected Ride activities grouped by month.
The `period` parameter accepts `day`, `week`, `month` or `year` to control
how statistics are grouped. Each response entry contains the period label,
ride count, total distance, average weighted power, average intensity
factor, cumulative training stress and average speed. Provide a comma
separated list of activity IDs through `ids` to limit the aggregation and
use the `types` parameter to restrict results to certain activity types.
Automated tools can refer to `AGENTS.md` for a short Python example.

Downloaded activities are organised as:

```
DATA_DIR/
  <user>/
    <year>/
      <id>/
        meta.json.zst
        streams.json.zst
    ftp.json
    weight.json
    wkg.json
    enduro.json
    fitness.json
```

Metadata and streams are encoded with `serde_json` and compressed using zstd. The `ftp.json` file stores Functional Threshold Power history used to compute IF and TSS. The `weight.json` file tracks weight changes, `wkg.json` records watts per kilogram and `enduro.json` and `fitness.json` keep the ride readiness scores over time.

## Adding Another User

The storage layout includes a `<user>` directory.  Specify the user name in your
`config.toml` and run a separate instance per athlete.  Each instance will store
its data under `DATA_DIR/<user>/`.

## Tests

Run the unit tests with:

```bash
cargo test
```

The integration tests link several large dependencies and may fail on machines
with limited memory. If you see the linker being killed (exit code `143`), try
running with:

```bash
RUSTFLAGS="-C link-arg=-Wl,--no-keep-memory" cargo test -- --test-threads=1
```

## Docker

The project includes a `Dockerfile` that builds a small container with the compiled binary. Build it locally with:

```bash
docker build -t abcy-data .
```

The container expects the environment variables described above at runtime.

### Continuous Integration

GitHub Actions run `cargo test` for each pull request and build the Docker image on every push to `main`.


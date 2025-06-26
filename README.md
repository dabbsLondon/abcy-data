# abcy-data

This project synchronizes cycling activities from Strava and exposes a simple HTTP API.  Activity metadata and data streams are stored as compressed JSON files for further analysis.

## Requirements

- **Rust** (edition 2021)
- A Strava account with API credentials

## Configuration

Configuration is read from `config.toml` instead of environment variables. A minimal example is provided in `config.example.toml`. Copy this file to `config.toml` and fill in your credentials:

```toml
[strava]
client_id = "12345"
client_secret = "secret"
# optional refresh token obtained during authorization
refresh_token = ""
token_path = "./token.json"      # cached access token and expiry

[storage]
data_dir = "./data"
download_count = 10
user = "athlete1"
```

If the cached token has expired or the Strava API returns a `401`, the
application automatically refreshes it using the stored `refresh_token`.
The new access token, refresh token and expiry time are written to `token_path`
and the expiry is logged. If no refresh token is provided the server will open
a browser on startup to complete the OAuth flow and store the returned
credentials.

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

  The JSON response contains `refresh_token` which can be stored in your
  configuration file. This step is optional because running the server with an
  empty `refresh_token` will trigger the interactive authorization flow.

## Running

```bash
cargo run
```

The `default-run` target is configured, so this command starts the main
`abcy-data` server without needing `--bin`.

On startup the app will:

1. Query your most recent activities (count configured by `download_count`).
2. Fetch metadata and data streams for each new activity and store them under
   `DATA_DIR/<user>/<year>/<id>/` as `meta.json.zst` and `streams.json.zst`.
3. Start an HTTP server on `localhost:8080`.

### API Endpoints

- `GET /activities` – ID, name, date and distance of all downloaded activities.
- `GET /activity/{id}` – full metadata and streams for an activity.
- `GET /files` – recursive listing of everything under `DATA_DIR`.
- `POST /webhook` – Strava webhook endpoint used to fetch new data immediately.

A Postman collection `abcy-data.postman_collection.json` is included to help
test the endpoints. Set the `base_url` variable to your server's address.
Automated tools can refer to `AGENTS.md` for a short Python example.

Downloaded activities are organised as:

```
DATA_DIR/
  <user>/
    <year>/
      <id>/
        meta.json.zst
        streams.json.zst
```

Metadata and streams are encoded with `serde_json` and compressed using zstd.

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


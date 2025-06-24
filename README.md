# abcy-data

This project synchronizes cycling activities from Strava, parses `.FIT` files, stores the results in Parquet and exposes a small HTTP API. It is intended as a foundation for further analysis services.

## Requirements

- **Rust** (edition 2021)
- A Strava account with API credentials

## Configuration

The application reads its settings from environment variables (or a `.env` file). The following variables are required:

```
STRAVA_CLIENT_ID=<client id>
STRAVA_CLIENT_SECRET=<client secret>
STRAVA_REFRESH_TOKEN=<refresh token>
DATA_DIR=./data                # directory for downloaded and processed files
```

`STRAVA_CLIENT_ID` and `STRAVA_CLIENT_SECRET` identify your Strava application.
`STRAVA_REFRESH_TOKEN` is obtained from the OAuth exchange described below and
is used to download activities on your behalf.
`DATA_DIR` points to the folder where raw `.fit` files and resulting Parquet
data will be stored.

Create a `.env` file in the project root with these variables so they are loaded automatically. An example file is provided as `.env.example` which you can copy and modify:

```bash
cp .env.example .env
# edit the file and add your values
```

The `.env` file is only used for local development. In production you may set the environment variables directly.

### Obtaining Strava Tokens

1. Create an application at <https://www.strava.com/settings/api> to receive a client ID and secret.
2. Authorize the application for your account by visiting a URL such as:

   ```text
   https://www.strava.com/oauth/authorize?client_id=<client id>&response_type=code&redirect_uri=http://localhost/exchange_token&approval_prompt=force&scope=activity:read_all
   ```

   After approving you will be redirected with a `code` query parameter.
3. Exchange that code for a long‑lived token via the Strava API:

   ```bash
   curl -X POST https://www.strava.com/oauth/token \
       -d client_id=<client id> \
       -d client_secret=<client secret> \
       -d code=<authorization code> \
       -d grant_type=authorization_code
   ```

   The JSON response contains `refresh_token` which should be stored in your `.env` file.
4. Populate the variables above with the values for your Strava account.

## Running

```bash
cargo run
```

On startup the app will:

1. Query your last ten Strava activities and download any that do not already have a corresponding Parquet file.
2. Parse each new file and write a `<activity_id>.parquet` file to `DATA_DIR`.
3. Begin checking for new activities every five minutes in the background.
4. Start an HTTP server on `localhost:8080`.

### API Endpoints

- `GET /activities` – list the IDs of stored activities.
- `GET /raw` – list the `.fit` files in `DATA_DIR`.

A Postman collection `abcy-data.postman_collection.json` is included to help
test the endpoints. Set the `base_url` variable to your server's address.
Automated tools can refer to `AGENTS.md` for a short Python example.

Each activity is stored as a Parquet file inside `DATA_DIR` and can be processed further using your preferred tools.

## Adding Another User

The current implementation expects one set of Strava credentials. To ingest data for another athlete you can run a second instance with a different `.env` file:

```bash
STRAVA_CLIENT_ID=... STRAVA_CLIENT_SECRET=... \
STRAVA_REFRESH_TOKEN=... DATA_DIR=./alice_data cargo run
```

Repeat for each user you want to track. A more advanced multi-user workflow would require extending the configuration and storage layout.

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


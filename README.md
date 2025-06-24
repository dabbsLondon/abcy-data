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

You can create a `.env` file in the project directory with these values so they are loaded automatically.

### Obtaining Strava Tokens

1. Create an application at <https://www.strava.com/settings/api> to receive a client ID and secret.
2. Use Strava's OAuth flow to exchange an authorization code for a long‑lived `refresh_token`.
3. Populate the variables above with the values for your Strava account.

## Running

```bash
cargo run
```

On startup the app will:

1. Query your Strava activities and download the latest one as a `.fit` file.
2. Parse the file and write a `<activity_id>.parquet` file to `DATA_DIR`.
3. Start an HTTP server on `localhost:8080`.

### API Endpoints

- `GET /activities` – list the IDs of stored activities.

Each activity is stored as a Parquet file inside `DATA_DIR` and can be processed further using your preferred tools.

## Adding Another User

The current implementation expects one set of Strava credentials. To ingest data for another athlete you can run a second instance with a different `.env` file:

```bash
STRAVA_CLIENT_ID=... STRAVA_CLIENT_SECRET=... \
STRAVA_REFRESH_TOKEN=... DATA_DIR=./alice_data cargo run
```

Repeat for each user you want to track. A more advanced multi-user workflow would require extending the configuration and storage layout.


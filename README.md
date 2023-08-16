Real time financial data streaming and resampling with Rust.

This crates is composed of two tasks continuously executing in parallel:

* Obtains Finnhub's realtime data and saves it to a PostgreSQL database in the table `trades`.
* Resamples data from `trades` into a predefined interval (by default, 100 milliseconds) and saves it on the table `resampled_trades`. Resampling is done by taking the lastest trade that happened prior to that instant.

Additionally an API to query the data is provided.

## Usage instructions without Docker compose

* Create a file called `.env` in the root directory (same folder that `LICENSE` is) with Finnhub API token, the Postgresql database url:

      FINNHUB_TOKEN=your_token_here
      POSTGRES_PASSWORD=your_postgresql_password_here
      DATABASE_URL=postgres://postgres:${POSTGRES_PASSWORD}@postgres/tplatform
      PGADMIN_DEFAULT_EMAIL=your@email.com
      PGADMIN_DEFAULT_PASSWORD=your_password_for_postgres_webadmin_here

* Run `docker compose up`.

* The API should became available at http://127.0.0.1:7500 while the API documentation will be at http://127.0.0.1:7500/docs and Postgres webadmin will be at http://127.0.0.1:7510

## Usage instructions without Docker compose

* Create a file called `.env` as explained in the previous sections (you will need to provide your own PostgreSQL server setup).

* If you don't have `Rust` installed, see `https://rustup.rs`

* Create the database with: `cargo install sqlx-cli && sqlx database create && sqlx migrate run`

* Start the data pipeline with `cargo run --bin rust-trading-platform-pipeline`

* Start the API with `cargo run --bin rust-trading-platform-api`

Real time financial data streaming and resampling with Rust.

This crates is composed of two tasks continuously executing in parallel:

* Obtains Finnhub's realtime data and saves it to a SQLite database in the table `trades`.
* Resamples data from `trades` into a predefined interval (by default, 100 milliseconds) and saves it on the table `resampled_trades`. Resampling is done by taking the lastest trade that happened prior to that instant.

## Usage instructions

* If you don't have `Rust` installed, see `https://rustup.rs`

* Create a file called `.env` in the root directory (same folder as `Cargo.toml`) with Finnhub API token and the database url:

      FINNHUB_TOKEN=your_token_here
      DATABASE_URL=sqlite://db.sqlite3

* Create the database with: `cargo install sqlx-cli && sqlx database create && sqlx migrate run`

* Then run the examples with `cargo run`

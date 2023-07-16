Using Finnhub real time data with Rust. This obtains Finnhub's realtime data and saves it to a SQLite database.

## Instructions

* If you don't have `Rust` installed, see `https://rustup.rs`.

* Create a file called `.env` in the root directory (same folder as `Cargo.toml`) with Finnhub API token and the database url:

    FINNHUB_TOKEN=your_token_here
    DATABASE_URL=sqlite://db.sqlite3

* Then run the examples with `cargo run`.

* Hint: consider using WAL mode for SQLite, see https://www.sqlite.org/wal.html

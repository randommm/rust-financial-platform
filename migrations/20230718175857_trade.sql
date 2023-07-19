-- Add migration script here
CREATE TABLE "trade" (
    "id" integer NOT NULL PRIMARY KEY AUTOINCREMENT,
    "price" real NOT NULL,
    "security" text NOT NULL,
    "timestamp" integer NOT NULL,
    "volume" real NOT NULL
);
CREATE TABLE "resampled_trade" (
    "id" integer NOT NULL PRIMARY KEY AUTOINCREMENT,
    "price" real NOT NULL,
    "security" text NOT NULL,
    "timestamp" integer NOT NULL,
    CONSTRAINT unique_resampled_trade UNIQUE(security, timestamp)
);
PRAGMA journal_mode=WAL;

-- Add migration script here
CREATE TABLE "trade" (
    "id" integer NOT NULL PRIMARY KEY AUTOINCREMENT,
    "price" real NOT NULL,
    "security" text NOT NULL,
    "timestamp" integer NOT NULL,
    "volume" real NOT NULL
);
PRAGMA journal_mode=WAL;

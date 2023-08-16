CREATE TABLE "trades_raw" (
    "id" BIGSERIAL NOT NULL PRIMARY KEY,
    "price" DOUBLE PRECISION NOT NULL,
    "security" TEXT NOT NULL,
    "timestamp" BIGINT NOT NULL,
    "volume" DOUBLE PRECISION NOT NULL
);
CREATE TABLE "trades_resampled" (
    "id" BIGSERIAL NOT NULL PRIMARY KEY,
    "price" DOUBLE PRECISION NOT NULL,
    "security" TEXT NOT NULL,
    "timestamp" BIGINT NOT NULL,
    CONSTRAINT unique_resampled_trade UNIQUE(security, timestamp)
);
CREATE INDEX trades_raw_index ON trades_raw(security, timestamp);
CREATE INDEX trades_resampled_index ON trades_resampled(security, timestamp);

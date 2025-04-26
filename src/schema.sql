-- ──────────────────────────────────────────────────────────────
--  schema.sql   (SQLite ≥3.38; JSON1 enabled)
-- ──────────────────────────────────────────────────────────────

PRAGMA foreign_keys = ON;
PRAGMA journal_mode = WAL;          -- safe concurrency
PRAGMA synchronous  = NORMAL;       -- durability-speed trade-off

/*───────────────────────────────────────────────────────────────*
 * 1) Current live snapshot  (single-row, fast REPLACE updates)  *
 *───────────────────────────────────────────────────────────────*/
CREATE TABLE IF NOT EXISTS portfolio_snapshots (
    id         TEXT PRIMARY KEY,      -- e.g. 'current'
    json_blob  TEXT NOT NULL,         -- whole Portfolio serialized (or BLOB if using bincode)
    updated_at INTEGER NOT NULL       -- Unix epoch seconds
);

/*───────────────────────────────────────────────────────────────*
 * 2) Optional history table  (one row per commit)               *
 *    Comment out if you don’t need daily audit trail.           *
 *───────────────────────────────────────────────────────────────*/
CREATE TABLE IF NOT EXISTS portfolio_history (
    ts         INTEGER  PRIMARY KEY,  -- Unix epoch seconds (to 1-sec granularity)
    json_blob  TEXT     NOT NULL
);
CREATE INDEX IF NOT EXISTS portfolio_history_idx ON portfolio_history(ts DESC);

/*───────────────────────────────────────────────────────────────*
 * 3) Locker for graceful shutdown flush (optional)              *
 *───────────────────────────────────────────────────────────────*/
CREATE TABLE IF NOT EXISTS schema_meta (
    key   TEXT PRIMARY KEY,
    value TEXT
);
INSERT OR IGNORE INTO schema_meta(key,value) VALUES ('version','1');


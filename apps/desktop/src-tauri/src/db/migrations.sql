PRAGMA journal_mode=WAL;
PRAGMA foreign_keys=ON;

CREATE TABLE IF NOT EXISTS source_item (
  id TEXT PRIMARY KEY,
  source TEXT NOT NULL,
  fetched_at INTEGER NOT NULL,
  seen_at INTEGER NOT NULL,
  payload_json TEXT NOT NULL,
  title TEXT,
  body TEXT,
  lat REAL,
  lon REAL,
  occurred_at INTEGER,
  hash TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS event (
  id TEXT PRIMARY KEY,
  first_seen INTEGER NOT NULL,
  last_seen INTEGER NOT NULL,
  title TEXT,
  summary TEXT,
  class TEXT,
  severity REAL,
  confidence REAL,
  lat REAL, lon REAL,
  bbox TEXT,
  geojson TEXT,
  source_rank INTEGER
);

CREATE TABLE IF NOT EXISTS ai_labels (
  event_id TEXT REFERENCES event(id) ON DELETE CASCADE,
  labels_json TEXT,
  severity REAL,
  PRIMARY KEY(event_id)
);

CREATE VIRTUAL TABLE IF NOT EXISTS event_fts USING fts5(title, summary, content='event', content_rowid='rowid');

CREATE TABLE IF NOT EXISTS alert (
  id TEXT PRIMARY KEY,
  source TEXT NOT NULL,
  headline TEXT,
  event TEXT,
  severity TEXT,
  urgency TEXT,
  certainty TEXT,
  onset INTEGER,
  sent INTEGER,
  expires INTEGER,
  area_desc TEXT,
  polygon_geojson TEXT,
  bbox_minx REAL, bbox_miny REAL, bbox_maxx REAL, bbox_maxy REAL,
  raw_json TEXT NOT NULL,
  last_seen INTEGER NOT NULL
);

CREATE VIRTUAL TABLE IF NOT EXISTS alert_rtree USING rtree(
  rowid, minx, maxx, miny, maxy
);

CREATE INDEX IF NOT EXISTS idx_ai_labels_event ON ai_labels(event_id);

PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS sessions (
  id TEXT PRIMARY KEY,
  mode TEXT NOT NULL CHECK (mode IN ('free', 'structured')),
  target_focus_ms INTEGER NOT NULL,
  status TEXT NOT NULL CHECK (
    status IN ('running', 'paused', 'completed', 'cancelled')
  ),
  started_at_ms INTEGER NOT NULL,
  ended_at_ms INTEGER,
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS session_steps (
  id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
  step_index INTEGER NOT NULL,
  kind TEXT NOT NULL CHECK (
    kind IN ('focus', 'short_break', 'long_break')
  ),
  planned_ms INTEGER NOT NULL,
  actual_ms INTEGER NOT NULL DEFAULT 0,
  status TEXT NOT NULL CHECK (
    status IN ('pending', 'running', 'paused', 'completed', 'skipped', 'cancelled')
  ),
  started_at_ms INTEGER,
  ended_at_ms INTEGER,
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL,
  UNIQUE(session_id, step_index)
);

CREATE TABLE IF NOT EXISTS time_intervals (
  id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
  step_id TEXT REFERENCES session_steps(id) ON DELETE SET NULL,
  kind TEXT NOT NULL CHECK (kind IN ('focus', 'break')),
  status TEXT NOT NULL CHECK (status IN ('open', 'closed')),
  started_at_ms INTEGER NOT NULL,
  ended_at_ms INTEGER,
  elapsed_ms INTEGER NOT NULL DEFAULT 0,
  stop_reason TEXT CHECK (
    stop_reason IN ('pause', 'complete', 'reset', 'quit', 'app_shutdown', 'step_switch')
  ),
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS settings (
  key TEXT PRIMARY KEY,
  value_json TEXT NOT NULL,
  updated_at_ms INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_time_intervals_started_at_ms
  ON time_intervals(started_at_ms);

CREATE INDEX IF NOT EXISTS idx_time_intervals_session_id
  ON time_intervals(session_id);

CREATE INDEX IF NOT EXISTS idx_session_steps_session_id
  ON session_steps(session_id);

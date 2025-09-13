-- Example D1 migration (optional)
-- You can apply with: wrangler d1 migrations apply DB

CREATE TABLE IF NOT EXISTS visits (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  chat_id TEXT,
  created_at TEXT DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS events(
  name TEXT NOT NULL,
  date TEXT PRIMARY KEY     NOT NULL,
  event_type TEXT NOT NULL,
  data_type TEXT NOT NULL,
  content TEXT NOT NULL,
  from_client TEXT NOT NULL
);
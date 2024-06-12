CREATE TABLE IF NOT EXISTS events(
  id TEXT  PRIMARY KEY     NOT NULL,
  name TEXT NOT NULL,
  date TEXT  NOT NULL,
  event_type TEXT NOT NULL,
  data_type TEXT NOT NULL,
  data_id TEXT NOT NULL,
  content TEXT NOT NULL,
  from_client TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS finalstates(
  id TEXT  PRIMARY KEY     NOT NULL,
  name TEXT NOT NULL,
  date TEXT  NOT NULL,
  data_type TEXT NOT NULL,
  content TEXT NOT NULL
);

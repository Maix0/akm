-- init.sql
CREATE TABLE IF NOT EXISTS clients (
  id INTEGER NOT NULL PRIMARY KEY ASC AUTOINCREMENT,
  name TEXT NOT NULL,
  description TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS keys (
  id INTEGER NOT NULL PRIMARY KEY ASC AUTOINCREMENT,
  name TEXT NOT NULL,
  description TEXT NOT NULL,
  -- nonce BLOB NOT NULL, -- used to encrypt/decrypt the above data
  apiKey TEXT, -- they actual api key
  rotateAt TEXT, -- try to autorotate the key at <DATE> where <DATE> is a `YYYY-MM-DD`
  rotateWith TEXT -- what to autorotate with
);

CREATE TABLE IF NOT EXISTS clients_key (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  clientID INTEGER NOT NULL, -- the client that will use this key
  keyID INTEGER NOT NULL, -- the key id
  secret TEXT NOT NULL, -- the secret that the client will need to provide
  lastUsed TEXT, -- unix timestamp
  --
  UNIQUE (clientID, keyID),
  FOREIGN KEY (clientID) REFERENCES clients (id),
  FOREIGN KEY (keyID) REFERENCES keys (id)
);

CREATE TABLE IF NOT EXISTS users (
  id INTEGER PRIMARY KEY ASC AUTOINCREMENT,
  name TEXT NOT NULL,
  token TEXT NOT NULL
);

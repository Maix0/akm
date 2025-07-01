-- init.sql
CREATE TABLE IF NOT EXISTS clients (
  id INTEGER PRIMARY KEY ASC AUTOINCREMENT,
  name TEXT NOT NULL,
  description TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS keys (
  id INTEGER PRIMARY KEY ASC AUTOINCREMENT,
  name TEXT NOT NULL,
  description TEXT NOT NULL,
  apiKey TEXT NOT NULL, -- they actual api key
  -- nonce BLOB NOT NULL, -- used to encrypt/decrypt the above data
  updateAt INTEGER, -- try to autoupdate the key at <DATE> where <DATE> is a unix timestamp
  updateWith TEXT, -- what to autoupdate with
  -- 
  -- either update* are BOTH null or not null, but they must be in sync
  CHECK (
    (
      updateAt IS NULL
      AND updateWith IS NULL
    )
    OR (
      updateAt IS NOT NULL
      AND updateWith IS NOT NULL
    )
  )
);

CREATE TABLE IF NOT EXISTS clients_key (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  clientID INTEGER NOT NULL, -- the client that will use this key
  keyID INTEGER NOT NULL, -- the key id
  secret TEXT NOT NULL, -- the secret that the client will need to provide
  lastUsed INTEGER, -- unix timestamp
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

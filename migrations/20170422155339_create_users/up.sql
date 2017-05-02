CREATE TABLE Users (
  id       SERIAL PRIMARY KEY,
  username CHAR(32) NOT NULL,
  password CHAR(64) NOT NULL,
  salt     CHAR(64) NOT NULL
);

CREATE UNIQUE INDEX username_index
  ON Users (username);

CREATE TABLE Sessions (
  key         CHAR(32) PRIMARY KEY          NOT NULL,
  user_id     INTEGER REFERENCES Users (id) NOT NULL,
  expire_date TIMESTAMP                     NOT NULL
);

CREATE UNIQUE INDEX session_key_index
  ON Sessions (key);
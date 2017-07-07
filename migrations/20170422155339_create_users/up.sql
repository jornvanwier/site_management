CREATE TABLE Organizations (
  id   SERIAL PRIMARY KEY,
  name CHAR(32) NOT NULL
);

CREATE TABLE Users (
  id              SERIAL PRIMARY KEY,
  username        CHAR(32)                              NOT NULL,
  password        CHAR(64)                              NOT NULL,
  salt            CHAR(64)                              NOT NULL,
  organization_id INTEGER REFERENCES Organizations (id) NOT NULL
);

CREATE UNIQUE INDEX username_index
  ON Users (username);

CREATE TABLE Sessions (
  key         CHAR(32) PRIMARY KEY,
  user_id     INTEGER REFERENCES Users (id) NOT NULL,
  expire_date TIMESTAMP                     NOT NULL
);

CREATE UNIQUE INDEX session_key_index
  ON Sessions (key);

CREATE TABLE Images (
  id       SERIAL PRIMARY KEY,
  user_id  INTEGER REFERENCES Users (id) NOT NULL,
  filename CHAR(32)                      NOT NULL
)
CREATE TABLE Websites (
  id   SERIAL PRIMARY KEY,
  name VARCHAR(32) NOT NULL
);

CREATE UNIQUE INDEX website_name_index
  ON Websites(name);

CREATE TABLE Users (
  id       SERIAL PRIMARY KEY,
  username VARCHAR(32) NOT NULL,
  password CHAR(64)    NOT NULL,
  salt     CHAR(64)    NOT NULL
);

CREATE UNIQUE INDEX username_index
  ON Users (username);

CREATE TABLE UserWebsites (
  user_id    INTEGER REFERENCES Users (id)    NOT NULL,
  website_id INTEGER REFERENCES Websites (id) NOT NULL,
  admin      BOOLEAN                          NOT NULL DEFAULT FALSE,
  PRIMARY KEY (user_id, website_id)
);

CREATE TABLE Sessions (
  key         CHAR(32) PRIMARY KEY,
  user_id     INTEGER REFERENCES Users (id) NOT NULL,
  expire_date TIMESTAMP                     NOT NULL
);

CREATE UNIQUE INDEX session_key_index
  ON Sessions (key);

CREATE TABLE Images (
  id          SERIAL PRIMARY KEY,
  website_id  INTEGER REFERENCES Websites (id)      NOT NULL,
  uploaded_by INTEGER REFERENCES Users (id)         NOT NULL,
  filename    VARCHAR(64)                           NOT NULL,
  upload_date TIMESTAMP                             NOT NULL
);

CREATE UNIQUE INDEX image_filename_index
  ON Images (filename);

CREATE TABLE Languages (
  name  VARCHAR(32) NOT NULL,
  short CHAR(2) PRIMARY KEY -- Lowercase
);

CREATE UNIQUE INDEX languages_name_index
  ON Languages (name);

INSERT INTO Languages VALUES ('English', 'en');
INSERT INTO Languages VALUES ('Dutch', 'nl');

CREATE TABLE Articles (
  id         SERIAL PRIMARY KEY,
  identifier VARCHAR(16)                      NOT NULL,
  website_id INTEGER REFERENCES Websites (id) NOT NULL,
  image_id   INTEGER REFERENCES Images (id) -- Can be null
);

CREATE UNIQUE INDEX articles_identifier_index
  ON Articles (identifier);

CREATE TABLE ArticleTexts (
  article_id INTEGER REFERENCES Articles (id)                  NOT NULL,
  lang_short CHAR(2)                                           NOT NULL,
  text       TEXT                                              NOT NULL,
  edit_date  TIMESTAMP                                         NOT NULL,
  edited_by  INTEGER REFERENCES Users (id)                     NOT NULL,
  PRIMARY KEY (article_id, lang_short)
);

CREATE INDEX article_texts_lang_index
  ON ArticleTexts (lang_short);
CREATE INDEX article_texts_article_index
  ON ArticleTexts (article_id);

CREATE TABLE Projects (
  id         SERIAL PRIMARY KEY,
  identifier VARCHAR(16)                      NOT NULL,
  website_id INTEGER REFERENCES Websites (id) NOT NULL,
  image_id   INTEGER REFERENCES Images (id)   NOT NULL -- Header image
);

CREATE TABLE ProjectTexts (
  project_id  INTEGER REFERENCES Projects (id)                      NOT NULL,
  lang_short  CHAR(2)                                               NOT NULL,
  description VARCHAR(250)                                          NOT NULL, -- limit description length
  full_text   TEXT                                                  NOT NULL,
  edit_date   TIMESTAMP                                             NOT NULL,
  edited_by   INTEGER REFERENCES Users (id)                         NOT NULL,
  PRIMARY KEY (project_id, lang_short)
);

-- Gallery
CREATE TABLE ProjectImages (
  project_id INTEGER REFERENCES Projects (id) NOT NULL,
  image_id   INTEGER REFERENCES Images (id)   NOT NULL,
  PRIMARY KEY (project_id, image_id)
);

CREATE INDEX project_images_project_index
  ON ProjectImages(project_id);
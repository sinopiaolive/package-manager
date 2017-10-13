CREATE TABLE files (
  namespace TEXT NOT NULL,
  name TEXT NOT NULL,
  data BYTEA NOT NULL,
  uploaded_on TIMESTAMP NOT NULL,
  PRIMARY KEY (namespace, name)
);

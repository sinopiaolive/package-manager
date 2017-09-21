CREATE TABLE packages (
  namespace TEXT NOT NULL,
  name TEXT NOT NULL,
  deleted TEXT,
  deleted_on TIMESTAMP,
  PRIMARY KEY (namespace, name)
);

CREATE TABLE package_owners (
  namespace TEXT NOT NULL,
  name TEXT NOT NULL,
  user_id TEXT NOT NULL,
  added_time TIMESTAMP NOT NULL DEFAULT NOW(),
  PRIMARY KEY (namespace, name, user_id),
  FOREIGN KEY (namespace, name) REFERENCES packages (namespace, name),
  FOREIGN KEY (user_id) REFERENCES users (id)
);

CREATE INDEX package_owners_by_package_id ON package_owners (namespace, name);
CREATE INDEX package_owners_by_user_id ON package_owners (user_id);

CREATE TABLE package_releases (
  namespace TEXT NOT NULL,
  name TEXT NOT NULL,
  version TEXT NOT NULL,
  publisher TEXT NOT NULL,
  publish_time TIMESTAMP NOT NULL DEFAULT NOW(),
  artifact_url TEXT NOT NULL,
  description TEXT NOT NULL,
  licence TEXT,
  licence_file TEXT,
  keywords TEXT[] NOT NULL DEFAULT '{}',
  manifest TEXT NOT NULL,
  readme TEXT,
  deprecated BOOLEAN NOT NULL DEFAULT FALSE,
  deprecated_by TEXT,
  deprecated_on TIMESTAMP,
  deleted TEXT,
  deleted_on TIMESTAMP,
  PRIMARY KEY (namespace, name, version),
  FOREIGN KEY (namespace, name) REFERENCES packages (namespace, name),
  FOREIGN KEY (publisher) REFERENCES users (id),
  FOREIGN KEY (deprecated_by) REFERENCES users (id)
);

CREATE INDEX package_releases_by_package_id ON package_releases (namespace, name);
CREATE INDEX package_releases_by_keyword ON package_releases (keywords);
CREATE INDEX package_releases_by_licence ON package_releases (licence);
CREATE INDEX package_releases_by_publisher ON package_releases (publisher);

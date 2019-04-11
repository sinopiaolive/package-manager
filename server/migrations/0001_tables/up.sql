-- Sessions

CREATE TABLE login_sessions (
  token TEXT PRIMARY KEY,
  callback TEXT NOT NULL,
  stamp TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE FUNCTION expire_login_sessions() RETURNS trigger
  LANGUAGE plpgsql
  AS $$
BEGIN
  DELETE FROM login_sessions WHERE stamp < NOW() - INTERVAL '30 minutes';
  RETURN NEW;
END;
$$;

CREATE TRIGGER expire_login_sessions_trigger
  AFTER INSERT ON login_sessions
  EXECUTE PROCEDURE expire_login_sessions();

-- Packages

CREATE TABLE users (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  email TEXT NOT NULL,
  avatar TEXT
);

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

  -- Metadata
  description TEXT NOT NULL,
  authors TEXT[] NOT NULL DEFAULT '{}',
  keywords TEXT[] NOT NULL DEFAULT '{}',
  homepage_url TEXT,
  repository_type TEXT,
  repository_url TEXT,
  bugs_url TEXT,

  license TEXT,
  license_file_name TEXT,
  license_file_contents TEXT,

  manifest_file_name TEXT,
  manifest_file_contents TEXT,

  readme_name TEXT,
  readme_contents TEXT,

  -- Bookkeeping
  publisher TEXT NOT NULL,
  publish_time TIMESTAMP NOT NULL DEFAULT NOW(),
  deleted TEXT,
  deleted_on TIMESTAMP,

  -- has_many release_dependencies
  -- has_many[sic!] files

  PRIMARY KEY (namespace, name, version),
  FOREIGN KEY (namespace, name) REFERENCES packages (namespace, name),
  FOREIGN KEY (publisher) REFERENCES users (id)
);

CREATE INDEX package_releases_by_package_id ON package_releases (namespace, name);
CREATE INDEX package_releases_by_keyword ON package_releases (keywords);
CREATE INDEX package_releases_by_license ON package_releases (license);
CREATE INDEX package_releases_by_publisher ON package_releases (publisher);

CREATE TABLE release_dependencies (
  namespace TEXT NOT NULL,
  name TEXT NOT NULL,
  version TEXT NOT NULL,
  ordering INTEGER NOT NULL,
  dependency_namespace TEXT NOT NULL,
  dependency_name TEXT NOT NULL,
  dependency_version_constraint TEXT NOT NULL,
  PRIMARY KEY (namespace, name, version, ordering),
  FOREIGN KEY (namespace, name, version) REFERENCES package_releases
);

CREATE TABLE files (
  -- id allows us to update data (e.g. for recompression) while keeping old
  -- revisions around. We always serve the highest (most recent) id for a given
  -- release.
  id BIGSERIAL PRIMARY KEY,
  namespace TEXT NOT NULL,
  name TEXT NOT NULL,
  version TEXT NOT NULL,
  data BYTEA NOT NULL,
  FOREIGN KEY (namespace, name, version) REFERENCES package_releases (namespace, name, version)
);
CREATE INDEX files_by_version ON files (namespace, name, version, id);

CREATE FUNCTION package_search(TEXT, TEXT[]) RETURNS TABLE(name TEXT)
  AS $$ SELECT name FROM (
    SELECT package_releases.namespace AS namespace,
           package_releases.name AS name,
           to_tsvector(package_releases.name) || to_tsvector(package_releases.description) AS document
        FROM package_releases
    ) p_search
    WHERE namespace = $1 AND document @@ to_tsquery(array_to_string($2, ' & '))
    GROUP BY namespace, name $$ LANGUAGE SQL;

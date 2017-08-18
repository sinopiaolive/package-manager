CREATE TABLE login_sessions (
  token VARCHAR PRIMARY KEY,
  callback VARCHAR NOT NULL,
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

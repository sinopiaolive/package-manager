CREATE FUNCTION package_search(TEXT, TEXT[]) RETURNS TABLE(name TEXT)
  AS $$ SELECT name FROM (
    SELECT package_releases.namespace AS namespace,
           package_releases.name AS name,
           to_tsvector(package_releases.name) || to_tsvector(package_releases.description) AS document
        FROM package_releases
    ) p_search
    WHERE namespace = $1 AND document @@ to_tsquery(array_to_string($2, ' & '))
    GROUP BY namespace, name $$ LANGUAGE SQL;

table! {
    files (id) {
        id -> Int8,
        namespace -> Text,
        name -> Text,
        version -> Text,
        data -> Bytea,
    }
}

table! {
    login_sessions (token) {
        token -> Text,
        callback -> Text,
        stamp -> Timestamp,
    }
}

table! {
    package_owners (namespace, name, user_id) {
        namespace -> Text,
        name -> Text,
        user_id -> Text,
        added_time -> Timestamp,
    }
}

table! {
    package_releases (namespace, name, version) {
        namespace -> Text,
        name -> Text,
        version -> Text,
        description -> Text,
        authors -> Array<Text>,
        keywords -> Array<Text>,
        homepage_url -> Nullable<Text>,
        repository_type -> Nullable<Text>,
        repository_url -> Nullable<Text>,
        bugs_url -> Nullable<Text>,
        license -> Nullable<Text>,
        license_file_name -> Nullable<Text>,
        license_file_contents -> Nullable<Text>,
        manifest_file_name -> Nullable<Text>,
        manifest_file_contents -> Nullable<Text>,
        readme_name -> Nullable<Text>,
        readme_contents -> Nullable<Text>,
        publisher -> Text,
        publish_time -> Timestamp,
        deleted -> Nullable<Text>,
        deleted_on -> Nullable<Timestamp>,
    }
}

table! {
    packages (namespace, name) {
        namespace -> Text,
        name -> Text,
        deleted -> Nullable<Text>,
        deleted_on -> Nullable<Timestamp>,
    }
}

table! {
    release_dependencies (namespace, name, version, dependency_namespace, dependency_name) {
        namespace -> Text,
        name -> Text,
        version -> Text,
        ordering -> Int4,
        dependency_namespace -> Text,
        dependency_name -> Text,
        dependency_version_constraint -> Text,
    }
}

table! {
    users (id) {
        id -> Text,
        name -> Text,
        email -> Text,
        avatar -> Nullable<Text>,
    }
}

joinable!(package_owners -> users (user_id));
joinable!(package_releases -> users (publisher));

allow_tables_to_appear_in_same_query!(
    files,
    login_sessions,
    package_owners,
    package_releases,
    packages,
    release_dependencies,
    users,
);

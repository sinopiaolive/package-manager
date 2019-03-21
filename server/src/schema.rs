table! {
    files (namespace, name) {
        namespace -> Text,
        name -> Text,
        version -> Text,
        tar_br -> Bytea,
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
        publisher -> Text,
        publish_time -> Timestamp,
        artifact_url -> Text,
        description -> Text,
        license -> Nullable<Text>,
        license_file -> Nullable<Text>,
        keywords -> Array<Text>,
        manifest -> Text,
        readme_filename -> Nullable<Text>,
        readme -> Nullable<Text>,
        deprecated -> Bool,
        deprecated_by -> Nullable<Text>,
        deprecated_on -> Nullable<Timestamp>,
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
    users (id) {
        id -> Text,
        name -> Text,
        email -> Text,
        avatar -> Nullable<Text>,
    }
}

joinable!(package_owners -> users (user_id));

allow_tables_to_appear_in_same_query!(
    files,
    login_sessions,
    package_owners,
    package_releases,
    packages,
    users,
);

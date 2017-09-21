// infer_schema!("dotenv:DATABASE_URL");

table! {
    users {
        id -> Text,
        name -> Text,
        email -> Text,
        avatar -> Nullable<Text>,
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
    packages (namespace, name) {
        namespace -> Text,
        name -> Text,
        deleted -> Nullable<Text>,
        deleted_on -> Nullable<Timestamp>,
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
    use diesel::types::*;
    use diesel::pg::types::sql_types::Array;

    package_releases (namespace, name, version) {
        namespace -> Text,
        name -> Text,
        version -> Text,
        publisher -> Text,
        publish_time -> Timestamp,
        artifact_url -> Text,
        description -> Text,
        licence -> Nullable<Text>,
        licence_file -> Nullable<Text>,
        keywords -> Array<Text>,
        manifest -> Text,
        readme -> Nullable<Text>,
        deprecated -> Bool,
        deprecated_by -> Nullable<Text>,
        deprecated_on -> Nullable<Timestamp>,
        deleted -> Nullable<Text>,
        deleted_on -> Nullable<Timestamp>,
    }
}

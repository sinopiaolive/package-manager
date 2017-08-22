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

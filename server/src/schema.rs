table! {
    users {
        id -> VarChar,
        name -> VarChar,
        email -> VarChar,
        avatar -> Nullable<VarChar>,
    }
}

table! {
    login_sessions (token) {
        token -> VarChar,
        callback -> VarChar,
        stamp -> Timestamp,
    }
}

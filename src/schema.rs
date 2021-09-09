table! {
    users (id) {
        id -> Uuid,
        email -> Varchar,
        name -> Nullable<Text>,
        created -> Timestamptz,
        last_login -> Timestamptz,
    }
}

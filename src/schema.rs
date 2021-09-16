table! {
    documents (id) {
        id -> Uuid,
        name -> Text,
        created -> Timestamptz,
        owner -> Nullable<Uuid>,
        mime -> Nullable<Text>,
        data -> Bytea,
        hash -> Nullable<Text>,
    }
}

table! {
    users (id) {
        id -> Uuid,
        email -> Varchar,
        name -> Nullable<Text>,
        created -> Timestamptz,
        last_login -> Nullable<Timestamptz>,
    }
}

joinable!(documents -> users (owner));

allow_tables_to_appear_in_same_query!(
    documents,
    users,
);

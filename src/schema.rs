table! {
    accesses (document_id, user_id) {
        document_id -> Uuid,
        user_id -> Uuid,
        created -> Timestamptz,
    }
}

table! {
    documents (id) {
        id -> Uuid,
        name -> Text,
        created -> Timestamptz,
        owner -> Uuid,
        mime -> Nullable<Text>,
        size -> Int8,
        data -> Bytea,
        hash -> Nullable<Text>,
    }
}

table! {
    limits (user_id) {
        user_id -> Uuid,
        max_documents -> Int4,
        max_size -> Int8,
        current_documents -> Int4,
        current_size -> Int8,
    }
}

table! {
    users (id) {
        id -> Uuid,
        email -> Varchar,
        name -> Text,
        created -> Timestamptz,
        last_login -> Nullable<Timestamptz>,
        site_admin -> Bool,
    }
}

joinable!(accesses -> documents (document_id));
joinable!(accesses -> users (user_id));
joinable!(documents -> users (owner));
joinable!(limits -> users (user_id));

allow_tables_to_appear_in_same_query!(
    accesses,
    documents,
    limits,
    users,
);

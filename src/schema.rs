table! {
    events (id) {
        id -> Varchar,
        name -> Varchar,
        description -> Nullable<Text>,
        longitude -> Float8,
        latitude -> Float8,
        date_time -> Timestamp,
    }
}

table! {
    user_events (id) {
        id -> Int4,
        user_id -> Varchar,
        event_id -> Varchar,
        owner -> Bool,
    }
}

table! {
    users (user_id) {
        user_id -> Varchar,
        email -> Varchar,
        password -> Varchar,
        jwt -> Nullable<Varchar>,
    }
}

joinable!(user_events -> events (event_id));
joinable!(user_events -> users (user_id));

allow_tables_to_appear_in_same_query!(
    events,
    user_events,
    users,
);

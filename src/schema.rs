
diesel::table! {
    posts (id) {
        id -> Nullable<Integer>,
        title -> Text,
        content -> Text,
        author -> Integer,
        timestamp -> BigInt,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        name -> Text,
        email -> Text,
        password_hash -> Nullable<Text>,
    }
}
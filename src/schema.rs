
diesel::table! {
    posts (id) {
        id -> Nullable<Integer>,
        title -> Text,
        content -> Text,
    }
}
// @generated automatically by Diesel CLI.

diesel::table! {
    aliases (id) {
        id -> Integer,
        alias -> Text,
        key -> Text,
        #[sql_name = "type"]
        type_ -> Text,
    }
}

diesel::table! {
    de (key) {
        key -> Text,
        name -> Text,
        tumonline_room_nr -> Nullable<Integer>,
        #[sql_name = "type"]
        type_ -> Text,
        type_common_name -> Text,
        lat -> Float,
        lon -> Float,
        data -> Text,
    }
}

diesel::table! {
    en (key) {
        key -> Text,
        name -> Text,
        tumonline_room_nr -> Nullable<Integer>,
        #[sql_name = "type"]
        type_ -> Text,
        type_common_name -> Text,
        lat -> Float,
        lon -> Float,
        data -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    aliases,
    de,
    en,
);

// @generated automatically by Diesel CLI.

diesel::table! {
    calendar (single_event_id) {
        key -> Varchar,
        tumonline_id -> Int4,
        dtstart -> Timestamp,
        dtend -> Timestamp,
        dtstamp -> Timestamp,
        event_id -> Int4,
        event_title -> Text,
        single_event_id -> Int4,
        single_event_type_id -> Text,
        single_event_type_name -> Text,
        event_type_id -> Text,
        event_type_name -> Nullable<Text>,
        course_type_name -> Nullable<Text>,
        course_type -> Nullable<Text>,
        course_code -> Nullable<Text>,
        course_semester_hours -> Nullable<Int4>,
        group_id -> Nullable<Text>,
        xgroup -> Nullable<Text>,
        status_id -> Text,
        status -> Text,
        comment -> Text,
        last_scrape -> Timestamp,
    }
}

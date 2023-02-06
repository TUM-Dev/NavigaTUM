// @generated automatically by Diesel CLI.

diesel::table! {
    calendar (single_event_id) {
        key -> Text,
        dtstart -> Timestamp,
        dtend -> Timestamp,
        dtstamp -> Timestamp,
        event_id -> Integer,
        event_title -> Text,
        single_event_id -> Integer,
        single_event_type_id -> Text,
        single_event_type_name -> Text,
        event_type_id -> Text,
        event_type_name -> Nullable<Text>,
        course_type_name -> Nullable<Text>,
        course_type -> Nullable<Text>,
        course_code -> Nullable<Text>,
        course_semester_hours -> Nullable<Integer>,
        group_id -> Nullable<Text>,
        xgroup -> Nullable<Text>,
        status_id -> Text,
        status -> Text,
        comment -> Text,
    }
}

diesel::table! {
    calendar_scrape (single_event_id) {
        key -> Text,
        dtstart -> Timestamp,
        dtend -> Timestamp,
        dtstamp -> Timestamp,
        event_id -> Integer,
        event_title -> Text,
        single_event_id -> Integer,
        single_event_type_id -> Text,
        single_event_type_name -> Text,
        event_type_id -> Text,
        event_type_name -> Nullable<Text>,
        course_type_name -> Nullable<Text>,
        course_type -> Nullable<Text>,
        course_code -> Nullable<Text>,
        course_semester_hours -> Nullable<Integer>,
        group_id -> Nullable<Text>,
        xgroup -> Nullable<Text>,
        status_id -> Text,
        status -> Text,
        comment -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(calendar, calendar_scrape);

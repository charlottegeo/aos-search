// @generated automatically by Diesel CLI.

diesel::table! {
    episodes (id) {
        id -> Nullable<Integer>,
        season_id -> Integer,
        number -> Integer,
        title -> Text,
    }
}

diesel::table! {
    lines (id) {
        id -> Nullable<Integer>,
        season_id -> Integer,
        episode_id -> Integer,
        speaker_id -> Nullable<Integer>,
        line_number -> Integer,
        content -> Text,
    }
}

diesel::table! {
    seasons (id) {
        id -> Nullable<Integer>,
        number -> Integer,
    }
}

diesel::table! {
    speakers (id) {
        id -> Nullable<Integer>,
        name -> Text,
    }
}

diesel::joinable!(episodes -> seasons (season_id));
diesel::joinable!(lines -> episodes (episode_id));
diesel::joinable!(lines -> seasons (season_id));
diesel::joinable!(lines -> speakers (speaker_id));

diesel::allow_tables_to_appear_in_same_query!(
    episodes,
    lines,
    seasons,
    speakers,
);

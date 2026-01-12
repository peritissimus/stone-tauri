// @generated automatically by Diesel CLI.

diesel::table! {
    attachments (id) {
        id -> Text,
        note_id -> Text,
        filename -> Text,
        mime_type -> Text,
        size -> Integer,
        path -> Text,
        created_at -> BigInt,
    }
}

diesel::table! {
    note_links (source_note_id, target_note_id) {
        source_note_id -> Text,
        target_note_id -> Text,
        created_at -> BigInt,
    }
}

diesel::table! {
    note_tags (note_id, tag_id) {
        note_id -> Text,
        tag_id -> Text,
        created_at -> BigInt,
    }
}

diesel::table! {
    note_topics (note_id, topic_id) {
        note_id -> Text,
        topic_id -> Text,
        confidence -> Nullable<Float>,
        is_manual -> Integer,
        created_at -> BigInt,
    }
}

diesel::table! {
    note_versions (id) {
        id -> Text,
        note_id -> Text,
        title -> Text,
        content -> Text,
        version_number -> Integer,
        created_at -> BigInt,
    }
}

diesel::table! {
    notebooks (id) {
        id -> Text,
        name -> Text,
        parent_id -> Nullable<Text>,
        workspace_id -> Nullable<Text>,
        folder_path -> Nullable<Text>,
        icon -> Nullable<Text>,
        color -> Nullable<Text>,
        position -> Nullable<Integer>,
        created_at -> BigInt,
        updated_at -> BigInt,
    }
}

diesel::table! {
    notes (id) {
        id -> Text,
        title -> Nullable<Text>,
        file_path -> Nullable<Text>,
        notebook_id -> Nullable<Text>,
        workspace_id -> Nullable<Text>,
        is_favorite -> Integer,
        is_pinned -> Integer,
        is_archived -> Integer,
        is_deleted -> Integer,
        deleted_at -> Nullable<BigInt>,
        embedding -> Nullable<Binary>,
        created_at -> BigInt,
        updated_at -> BigInt,
    }
}

diesel::table! {
    settings (key) {
        key -> Text,
        value -> Text,
        updated_at -> BigInt,
    }
}

diesel::table! {
    tags (id) {
        id -> Text,
        name -> Text,
        color -> Nullable<Text>,
        created_at -> BigInt,
        updated_at -> BigInt,
    }
}

diesel::table! {
    topics (id) {
        id -> Text,
        name -> Text,
        description -> Nullable<Text>,
        color -> Nullable<Text>,
        is_predefined -> Integer,
        centroid -> Nullable<Binary>,
        note_count -> Nullable<Integer>,
        created_at -> BigInt,
        updated_at -> BigInt,
    }
}

diesel::table! {
    workspaces (id) {
        id -> Text,
        name -> Text,
        folder_path -> Text,
        is_active -> Integer,
        created_at -> BigInt,
        last_accessed_at -> BigInt,
    }
}

diesel::joinable!(attachments -> notes (note_id));
diesel::joinable!(note_tags -> notes (note_id));
diesel::joinable!(note_tags -> tags (tag_id));
diesel::joinable!(note_topics -> notes (note_id));
diesel::joinable!(note_topics -> topics (topic_id));
diesel::joinable!(note_versions -> notes (note_id));
diesel::joinable!(notebooks -> workspaces (workspace_id));
diesel::joinable!(notes -> notebooks (notebook_id));
diesel::joinable!(notes -> workspaces (workspace_id));

diesel::allow_tables_to_appear_in_same_query!(
    attachments,
    note_links,
    note_tags,
    note_topics,
    note_versions,
    notebooks,
    notes,
    settings,
    tags,
    topics,
    workspaces,
);

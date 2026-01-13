#[cfg(test)]
mod note_path_tests {
    /// Test path normalization logic that's used in get_note_by_path command
    fn normalize_file_path(file_path: &str, workspace_path: &str) -> String {
        // Normalize both paths to not have leading/trailing slashes for comparison
        let workspace_normalized = workspace_path.trim_start_matches('/').trim_end_matches('/');
        let file_normalized = file_path.trim_start_matches('/');

        // Try to strip workspace path prefix
        if let Some(relative) = file_normalized.strip_prefix(workspace_normalized) {
            relative.trim_start_matches('/').to_string()
        } else {
            file_path.trim_start_matches('/').to_string()
        }
    }

    #[test]
    fn test_path_normalization_with_leading_slash() {
        let workspace = "/Users/peritissimus/NoteBook";
        let file_path = "/Users/peritissimus/NoteBook/Work/Infra Release Bugs.md";
        let result = normalize_file_path(file_path, workspace);
        assert_eq!(result, "Work/Infra Release Bugs.md");
    }

    #[test]
    fn test_path_normalization_without_leading_slash() {
        let workspace = "/Users/peritissimus/NoteBook";
        let file_path = "Users/peritissimus/NoteBook/Work/Infra Release Bugs.md";
        let result = normalize_file_path(file_path, workspace);
        assert_eq!(result, "Work/Infra Release Bugs.md");
    }

    #[test]
    fn test_path_normalization_already_relative() {
        let workspace = "/Users/peritissimus/NoteBook";
        let file_path = "Work/Infra Release Bugs.md";
        let result = normalize_file_path(file_path, workspace);
        assert_eq!(result, "Work/Infra Release Bugs.md");
    }

    #[test]
    fn test_path_normalization_different_workspace() {
        let workspace = "/Users/other/Documents";
        let file_path = "Users/peritissimus/NoteBook/Work/Infra Release Bugs.md";
        let result = normalize_file_path(file_path, workspace);
        // Should return as-is since it doesn't match workspace
        assert_eq!(result, "Users/peritissimus/NoteBook/Work/Infra Release Bugs.md");
    }
}

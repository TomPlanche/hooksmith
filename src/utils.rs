/// Helper function to print an error message with a consistent format
pub fn print_error(title: &str, details: &str, suggestion: &str) {
    eprintln!("❌ ERROR: {title}\n\n{details}\n\n{suggestion}");
}

/// Helper function to print a warning message with a consistent format
pub fn print_warning(title: &str, details: &str) {
    println!("⚠️ WARNING: {title}\n\n{details}");
}

/// Helper function to print a success message with a consistent format
pub fn print_success(title: &str, details: &str) {
    println!("✅ SUCCESS: {title}\n\n{details}");
}

/// Helper function to format a list of items for display
pub fn format_list<T: std::fmt::Display>(items: &[T]) -> String {
    items
        .iter()
        .map(|item| format!("  - {item}"))
        .collect::<Vec<_>>()
        .join("\n")
}

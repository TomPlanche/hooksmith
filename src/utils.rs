use std::fmt::Display;

/// Trait for message types.
trait MessageType {
    /// The emoji prefix for each message type (e.g., "🚨 ERROR")
    const PREFIX: &'static str;

    /// Whether to output to stderr (true) or stdout (false)
    const TO_STDERR: bool = false;
}

// Define the message types
struct Error;
struct Warning;
struct Success;

// Implement the MessageType trait for each type
impl MessageType for Error {
    const PREFIX: &'static str = "🚨 ERROR";
    const TO_STDERR: bool = true;
}

impl MessageType for Warning {
    const PREFIX: &'static str = "⚠️ WARNING";
}

impl MessageType for Success {
    const PREFIX: &'static str = "✅ SUCCESS";
}

/// Formats a message without suggestion.
///
/// # Arguments
/// * `title` - The title of the message.
/// * `details` - The details of the message.
///
/// # Returns
/// * String - The formatted message.
fn format_message<T: MessageType>(title: &str, details: &str) -> String {
    format!("{}: {title}\n\n{details}", T::PREFIX)
}

/// Formats a message with suggestion.
///
/// # Arguments
/// * `title` - The title of the message.
/// * `details` - The details of the message.
/// * `suggestion` - The suggestion for the message.
///
/// # Returns
/// * String - The formatted message.
fn format_message_with_suggestion<T: MessageType>(
    title: &str,
    details: &str,
    suggestion: &str,
) -> String {
    format!("{}\n\n{suggestion}", format_message::<T>(title, details))
}

/// Prints a message without suggestion.
///
/// # Arguments
/// * `title` - The title of the message.
/// * `details` - The details of the message.
///
/// # Returns
/// * String - The formatted message.
fn print_message<T: MessageType>(title: &str, details: &str) {
    let message = format_message::<T>(title, details);

    if T::TO_STDERR {
        eprintln!("{message}");
    } else {
        println!("{message}");
    }
}

/// Prints a message with suggestion.
///
/// # Arguments
/// * `title` - The title of the message.
/// * `details` - The details of the message.
/// * `suggestion` - The suggestion for resolving the message.
///
/// # Returns
/// * String - The formatted message.
fn print_message_with_suggestion<T: MessageType>(title: &str, details: &str, suggestion: &str) {
    let message = format_message_with_suggestion::<T>(title, details, suggestion);
    if T::TO_STDERR {
        eprintln!("{message}");
    } else {
        println!("{message}");
    }
}

/// Prints an error message with a consistent format for user-friendly display.
///
/// # Arguments
/// - `title`: The title of the error message.
/// - `details`: The details of the error message.
/// - `suggestion`: The suggestion for resolving the error.
pub fn print_error(title: &str, details: &str, suggestion: &str) {
    print_message_with_suggestion::<Error>(title, details, suggestion);
}

/// Prints a warning message with a consistent format for user-friendly display.
///
/// # Arguments
/// - `title`: The title of the warning message.
/// - `details`: The details of the warning message.
pub fn print_warning(title: &str, details: &str) {
    print_message::<Warning>(title, details);
}

/// Prints a success message with a consistent format for user-friendly display.
///
/// # Arguments
/// - `title`: The title of the success message.
/// - `details`: The details of the success message.
pub fn print_success(title: &str, details: &str) {
    print_message::<Success>(title, details);
}

/// Formats a list of items with a consistent format for user-friendly display.
///
/// # Arguments
/// - `items`: The list of items to format.
///
/// # Returns
/// * String - A formatted string representation of the list.
pub fn format_list<T: Display>(items: &[T]) -> String {
    items
        .iter()
        .map(|item| format!("  - {item}"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_message() {
        let title = "Test Title";
        let details = "Test Details";

        // Test error message
        let error_msg = format_message::<Error>(title, details);
        assert!(error_msg.contains("🚨 ERROR"));
        assert!(error_msg.contains(title));
        assert!(error_msg.contains(details));

        // Test warning message
        let warning_msg = format_message::<Warning>(title, details);
        assert!(warning_msg.contains("⚠️ WARNING"));
        assert!(warning_msg.contains(title));
        assert!(warning_msg.contains(details));

        // Test success message
        let success_msg = format_message::<Success>(title, details);
        assert!(success_msg.contains("✅ SUCCESS"));
        assert!(success_msg.contains(title));
        assert!(success_msg.contains(details));
    }

    #[test]
    fn test_format_message_with_suggestion() {
        let title = "Test Title";
        let details = "Test Details";
        let suggestion = "Test Suggestion";

        // Test error message with suggestion
        let error_msg = format_message_with_suggestion::<Error>(title, details, suggestion);
        assert!(error_msg.contains("🚨 ERROR"));
        assert!(error_msg.contains(title));
        assert!(error_msg.contains(details));
        assert!(error_msg.contains(suggestion));

        // Test warning message with suggestion
        let warning_msg = format_message_with_suggestion::<Warning>(title, details, suggestion);
        assert!(warning_msg.contains("⚠️ WARNING"));
        assert!(warning_msg.contains(title));
        assert!(warning_msg.contains(details));
        assert!(warning_msg.contains(suggestion));

        // Test success message with suggestion
        let success_msg = format_message_with_suggestion::<Success>(title, details, suggestion);
        assert!(success_msg.contains("✅ SUCCESS"));
        assert!(success_msg.contains(title));
        assert!(success_msg.contains(details));
        assert!(success_msg.contains(suggestion));
    }

    #[test]
    fn test_format_list() {
        let empty_list: Vec<String> = vec![];
        assert_eq!(format_list(&empty_list), "");

        let single_item = vec!["item1".to_string()];
        assert_eq!(format_list(&single_item), "  - item1");

        let multiple_items = vec!["item1".to_string(), "item2".to_string()];
        let formatted = format_list(&multiple_items);
        assert!(formatted.contains("  - item1"));
        assert!(formatted.contains("  - item2"));
    }
}

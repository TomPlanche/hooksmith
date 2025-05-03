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

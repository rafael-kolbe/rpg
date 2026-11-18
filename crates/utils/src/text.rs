//! String format checks.

/// Whether `text` is lowercase kebab-case: ASCII lowercase letters and digits
/// separated by single hyphens, with no hyphen at either end.
///
/// # Examples
///
/// ```
/// use utils::text::is_kebab_case;
///
/// assert!(is_kebab_case("iron-sword"));
/// assert!(is_kebab_case("fire-2"));
/// assert!(!is_kebab_case("Iron-Sword"));
/// assert!(!is_kebab_case("iron_sword"));
/// assert!(!is_kebab_case("-iron"));
/// assert!(!is_kebab_case("iron--sword"));
/// assert!(!is_kebab_case(""));
/// ```
#[must_use]
pub fn is_kebab_case(text: &str) -> bool {
    !text.is_empty()
        && !text.starts_with('-')
        && !text.ends_with('-')
        && !text.contains("--")
        && text
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

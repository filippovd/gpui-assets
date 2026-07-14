//! Shared runtime utilities for the `gpui-assets` workspace.
//!
//! This crate holds small helpers that are needed by multiple workspace
//! members but do not belong to any single crate. It deliberately stays
//! dependency-light.

/// Convert an SVG file name (e.g. `arrow-right.svg`) into PascalCase.
///
/// This logic must stay in sync with the `pascal_case` helper inside
/// `gpui-assets-macros`, because the proc-macro generates enum variant names
/// from the same file names at compile time.
///
/// # Examples
///
/// ```
/// use gpui_assets_utils::pascal_case_name;
///
/// assert_eq!(pascal_case_name("arrow-right.svg"), "ArrowRight");
/// assert_eq!(pascal_case_name("some_icon_name.svg"), "SomeIconName");
/// assert_eq!(pascal_case_name("icon-123.svg"), "Icon123");
/// ```
pub fn pascal_case_name(filename: &str) -> String {
    let name = filename.strip_suffix(".svg").unwrap_or(filename);
    name.split(['-', '_', '.'])
        .filter(|part| !part.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) if first.is_ascii_digit() => word.to_string(),
                Some(first) => {
                    let mut result = String::with_capacity(word.len());
                    result.extend(first.to_uppercase());
                    result.push_str(chars.as_str().to_lowercase().as_str());
                    result
                }
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handles_dashes_underscores_dots() {
        assert_eq!(pascal_case_name("arrow-right.svg"), "ArrowRight");
        assert_eq!(pascal_case_name("some_icon_name.svg"), "SomeIconName");
        assert_eq!(pascal_case_name("icon.with.dots.svg"), "IconWithDots");
    }

    #[test]
    fn preserves_leading_digits() {
        assert_eq!(pascal_case_name("icon-123.svg"), "Icon123");
        assert_eq!(pascal_case_name("123icon.svg"), "123icon");
    }

    #[test]
    fn collapses_multiple_separators() {
        assert_eq!(pascal_case_name("a--b__c.d.svg"), "ABCD");
    }

    #[test]
    fn works_without_extension() {
        assert_eq!(pascal_case_name("arrow-right"), "ArrowRight");
    }
}

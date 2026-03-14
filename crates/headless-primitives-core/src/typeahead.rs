//! Low-level typeahead matching helpers.
//!
//! Matching is Unicode-aware through standard lowercase mappings.
//! It does not perform Unicode normalization or full case folding.

/// Returns the index of the first item whose label starts with `query`.
///
/// Matching is Unicode-aware and case-insensitive through lowercase mappings.
/// It returns `None` for empty queries.
pub fn typeahead_match<T>(items: &[T], query: &str, label: impl Fn(&T) -> &str) -> Option<usize> {
    if query.is_empty() {
        return None;
    }

    items.iter().position(|item| {
        let value = label(item);
        starts_with_ignore_case(value, query)
    })
}

fn starts_with_ignore_case(value: &str, query: &str) -> bool {
    let mut value_chars = value.chars().flat_map(char::to_lowercase);
    let mut query_chars = query.chars().flat_map(char::to_lowercase);

    loop {
        let Some(query_char) = query_chars.next() else {
            return true;
        };
        let Some(value_char) = value_chars.next() else {
            return false;
        };
        if value_char != query_char {
            return false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::typeahead_match;

    #[test]
    fn typeahead_matches_prefix() {
        let items = ["Apple", "Apricot", "Banana"];
        let index = typeahead_match(&items, "ap", |item| item);
        assert_eq!(index, Some(0));
    }

    #[test]
    fn typeahead_is_case_insensitive() {
        let items = ["Apple", "Banana"];
        let index = typeahead_match(&items, "bA", |item| item);
        assert_eq!(index, Some(1));
    }

    #[test]
    fn typeahead_matches_non_ascii_prefixes_case_insensitively() {
        let items = ["Ångström", "Éclair", "Banana"];
        assert_eq!(typeahead_match(&items, "ång", |item| item), Some(0));
        assert_eq!(typeahead_match(&items, "éC", |item| item), Some(1));
    }

    #[test]
    fn typeahead_handles_lowercase_expansions() {
        let items = ["İstanbul", "Izmir"];
        let index = typeahead_match(&items, "i̇s", |item| item);
        assert_eq!(index, Some(0));
    }

    #[test]
    fn typeahead_returns_none_for_empty_query() {
        let items = ["Apple"];
        let index = typeahead_match(&items, "", |item| item);
        assert_eq!(index, None);
    }
}

//! Low-level typeahead matching helpers.
//!
//! Matching is Unicode-aware through standard lowercase mappings.
//! It does not perform Unicode normalization or full case folding.

use alloc::string::String;

pub const TYPEAHEAD_DEFAULT_TIMEOUT_MS: u64 = 1_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeaheadKeyResult {
    Accepted,
    Ignored,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Typeahead {
    query: String,
    timeout_ms: u64,
    last_input_ms: Option<u64>,
}

impl Typeahead {
    pub fn new() -> Self {
        Self::with_timeout_ms(TYPEAHEAD_DEFAULT_TIMEOUT_MS)
    }

    pub fn with_timeout_ms(timeout_ms: u64) -> Self {
        Self {
            query: String::new(),
            timeout_ms,
            last_input_ms: None,
        }
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn timeout_ms(&self) -> u64 {
        self.timeout_ms
    }

    pub fn clear(&mut self) {
        self.query.clear();
        self.last_input_ms = None;
    }

    pub fn input(&mut self, key: &str, now_ms: u64) -> TypeaheadKeyResult {
        if !typeahead_key_is_searchable(key) {
            return TypeaheadKeyResult::Ignored;
        }

        if self.should_reset(now_ms) {
            self.query.clear();
        }
        self.query.push_str(key);
        self.last_input_ms = Some(now_ms);
        TypeaheadKeyResult::Accepted
    }

    pub fn match_index<T>(
        &self,
        items: &[T],
        current: Option<usize>,
        disabled: impl Fn(usize, &T) -> bool,
        label: impl Fn(&T) -> &str,
    ) -> Option<usize> {
        typeahead_match_from(items, self.query(), current, disabled, label)
    }

    fn should_reset(&self, now_ms: u64) -> bool {
        self.last_input_ms
            .is_some_and(|last_input_ms| now_ms.saturating_sub(last_input_ms) > self.timeout_ms)
    }
}

impl Default for Typeahead {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns the index of the first item whose label starts with `query`.
///
/// Matching is Unicode-aware and case-insensitive through lowercase mappings.
/// It returns `None` for empty queries.
pub fn typeahead_match<T>(items: &[T], query: &str, label: impl Fn(&T) -> &str) -> Option<usize> {
    typeahead_match_from(items, query, None, |_, _| false, label)
}

/// Returns the next matching item, starting after `current` when provided.
pub fn typeahead_match_from<T>(
    items: &[T],
    query: &str,
    current: Option<usize>,
    disabled: impl Fn(usize, &T) -> bool,
    label: impl Fn(&T) -> &str,
) -> Option<usize> {
    if query.is_empty() {
        return None;
    }

    let query = typeahead_effective_query(query);
    let start = current
        .map(|index| index.saturating_add(1))
        .unwrap_or(0)
        .min(items.len());

    (start..items.len()).chain(0..start).find(|index| {
        let item = &items[*index];
        !disabled(*index, item) && starts_with_ignore_case(label(item), &query)
    })
}

fn typeahead_key_is_searchable(key: &str) -> bool {
    let mut chars = key.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    chars.next().is_none() && !first.is_control()
}

fn typeahead_effective_query(query: &str) -> String {
    let mut chars = query.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    if chars.all(|ch| ch == first) {
        let mut value = String::new();
        value.push(first);
        value
    } else {
        String::from(query)
    }
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
    use super::{
        TYPEAHEAD_DEFAULT_TIMEOUT_MS, Typeahead, TypeaheadKeyResult, typeahead_match,
        typeahead_match_from,
    };

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

    #[test]
    fn typeahead_buffers_keys_until_timeout() {
        let mut typeahead = Typeahead::new();
        assert_eq!(typeahead.timeout_ms(), TYPEAHEAD_DEFAULT_TIMEOUT_MS);
        assert_eq!(typeahead.input("A", 0), TypeaheadKeyResult::Accepted);
        assert_eq!(typeahead.query(), "A");
        assert_eq!(typeahead.input("p", 500), TypeaheadKeyResult::Accepted);
        assert_eq!(typeahead.query(), "Ap");
        assert_eq!(typeahead.input("B", 1_501), TypeaheadKeyResult::Accepted);
        assert_eq!(typeahead.query(), "B");
    }

    #[test]
    fn typeahead_ignores_non_search_keys() {
        let mut typeahead = Typeahead::new();
        assert_eq!(typeahead.input("ArrowDown", 0), TypeaheadKeyResult::Ignored);
        assert_eq!(typeahead.input("", 0), TypeaheadKeyResult::Ignored);
        assert_eq!(typeahead.input("\u{7f}", 0), TypeaheadKeyResult::Ignored);
        assert_eq!(typeahead.query(), "");
    }

    #[test]
    fn typeahead_repeated_keys_cycle_from_current_index() {
        let items = ["Apple", "Apricot", "Avocado"];
        let mut typeahead = Typeahead::new();
        typeahead.input("A", 0);
        assert_eq!(
            typeahead.match_index(&items, Some(0), |_, _| false, |item| item),
            Some(1)
        );
        typeahead.input("A", 100);
        assert_eq!(
            typeahead.match_index(&items, Some(1), |_, _| false, |item| item),
            Some(2)
        );
    }

    #[test]
    fn typeahead_search_skips_disabled_items() {
        let items = ["Apple", "Apricot", "Banana"];
        assert_eq!(
            typeahead_match_from(&items, "a", Some(0), |index, _| index == 1, |item| item),
            Some(0)
        );
    }

    #[test]
    fn typeahead_search_starts_after_current_index() {
        let items = ["Apple", "Apricot", "Banana"];
        assert_eq!(
            typeahead_match_from(&items, "a", Some(0), |_, _| false, |item| item),
            Some(1)
        );
    }

    #[test]
    fn typeahead_search_handles_empty_labels() {
        let items = ["", "Banana"];
        assert_eq!(
            typeahead_match_from(&items, "b", None, |_, _| false, |item| item),
            Some(1)
        );
    }
}

pub fn typeahead_match<T>(
    items: &[T],
    query: &str,
    label: impl Fn(&T) -> &str,
) -> Option<usize> {
    if query.is_empty() {
        return None;
    }

    items.iter().position(|item| {
        let value = label(item);
        starts_with_ignore_ascii_case(value, query)
    })
}

fn starts_with_ignore_ascii_case(value: &str, query: &str) -> bool {
    let mut value_bytes = value.bytes();
    for query_byte in query.bytes() {
        let Some(value_byte) = value_bytes.next() else {
            return false;
        };
        if value_byte.to_ascii_lowercase() != query_byte.to_ascii_lowercase() {
            return false;
        }
    }
    true
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
    fn typeahead_returns_none_for_empty_query() {
        let items = ["Apple"];
        let index = typeahead_match(&items, "", |item| item);
        assert_eq!(index, None);
    }
}

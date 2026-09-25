fn asset_match_score(query: &str, name: &str) -> Option<u32> {
    let query = query.trim().to_lowercase();
    let name = name.to_lowercase();

    if query.is_empty() {
        return None;
    }

    if name == query {
        return Some(0);
    }

    if name.starts_with(&query) {
        return Some(10);
    }

    if name.contains(&query) {
        return Some(20);
    }

    let query_chars: Vec<char> = query.chars().collect();
    let name_chars: Vec<char> = name.chars().collect();

    let mut query_index = 0;

    for character in name_chars {
        if query_index < query_chars.len() && character == query_chars[query_index] {
            query_index += 1;
        }
    }

    (query_index == query_chars.len()).then_some(30)
}

pub fn ranked_asset_matches<I>(names: I, query: &str, limit: usize) -> Vec<String>
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    let mut matches: Vec<(u32, String)> = names
        .into_iter()
        .filter_map(|name| {
            let name = name.as_ref();

            asset_match_score(query, name).map(|score| (score, name.to_string()))
        })
        .collect();

    matches.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

    matches
        .into_iter()
        .take(limit)
        .map(|(_, name)| name)
        .collect()
}

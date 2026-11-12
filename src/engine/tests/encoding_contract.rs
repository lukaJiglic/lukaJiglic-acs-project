//! Spec 006, criterion 1: the sizes and ranges the engine exports equal the
//! tables of `docs/encoding.md`. The test reads the document.

use std::collections::BTreeMap;
use std::path::Path;

use tarok_engine::encode;

fn document() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/encoding.md");
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

/// All Markdown tables of the document, as rows of cells. The rule line
/// under a header is left out.
fn tables(text: &str) -> Vec<Vec<Vec<String>>> {
    let mut tables = Vec::new();
    let mut rows: Vec<Vec<String>> = Vec::new();
    for line in text.lines() {
        if line.starts_with('|') {
            let cells: Vec<String> = line
                .trim()
                .trim_matches('|')
                .split('|')
                .map(|cell| cell.trim().to_string())
                .collect();
            let rule = cells
                .iter()
                .all(|cell| cell.chars().all(|c| "-: ".contains(c)));
            if !rule {
                rows.push(cells);
            }
        } else if !rows.is_empty() {
            tables.push(std::mem::take(&mut rows));
        }
    }
    if !rows.is_empty() {
        tables.push(rows);
    }
    tables
}

/// The rows under the table whose header starts with these cells.
fn table_with<'a>(tables: &'a [Vec<Vec<String>>], header: &[&str]) -> &'a [Vec<String>] {
    for table in tables {
        let head: Vec<String> = table[0].iter().map(|cell| cell.to_lowercase()).collect();
        if head.len() >= header.len()
            && header
                .iter()
                .zip(&head)
                .all(|(want, have)| want.to_lowercase() == *have)
        {
            return &table[1..];
        }
    }
    panic!("no table with the header {header:?} in docs/encoding.md");
}

/// The whole numbers in a cell.
fn numbers(cell: &str) -> Vec<usize> {
    let mut found = Vec::new();
    let mut current = String::new();
    for c in cell.chars().chain(std::iter::once(' ')) {
        if c.is_ascii_digit() {
            current.push(c);
        } else if !current.is_empty() {
            found.push(current.parse().unwrap());
            current.clear();
        }
    }
    found
}

/// "2–14" is (2, 13); "88" is (88, 1): the first number and the count.
fn span(cell: &str) -> (usize, usize) {
    let found = numbers(cell);
    let first = found[0];
    let last = *found.last().unwrap();
    (first, last - first + 1)
}

/// "`BID_x`" is "BID", "`ANNOUNCE_BARVNI_VALAT`*" is "ANNOUNCE_BARVNI_VALAT".
fn group_name(cell: &str) -> String {
    let mut name = cell.replace(['`', '*'], "");
    for suffix in ["_x", "_suit", "_card", "_group", "_target"] {
        if let Some(stripped) = name.strip_suffix(suffix) {
            name = stripped.to_string();
        }
    }
    if name == "reserved" {
        name = "RESERVED".to_string();
    }
    name
}

#[test]
fn c01_exported_sizes_equal_the_tables_of_encoding_md() {
    let text = document();
    let tables = tables(&text);

    // The version.
    let marker = "Encoding version: **";
    let at = text.find(marker).expect("the version line") + marker.len();
    let version: u32 = text[at..].split('*').next().unwrap().parse().unwrap();
    assert_eq!(version, encode::ENCODING_VERSION);

    // Section 2: the arrays of one decision.
    let arrays: BTreeMap<String, String> = table_with(&tables, &["Array", "Shape"])
        .iter()
        .map(|row| (row[0].replace('`', ""), row[1].clone()))
        .collect();
    assert_eq!(
        numbers(&arrays["tokens"]),
        [encode::MAX_TOKENS, encode::TOKEN_FIELDS]
    );
    assert_eq!(numbers(&arrays["state"]), [encode::STATE_FEATURES]);
    assert_eq!(numbers(&arrays["mask"]), [encode::N_ACTIONS]);
    assert_eq!(
        numbers(&arrays["critic"]),
        [encode::STATE_FEATURES, encode::CRITIC_EXTRA]
    );
    assert_eq!(
        (
            encode::N_EVENTS,
            encode::STATE_FEATURES,
            encode::N_ACTIONS,
            encode::MAX_TOKENS,
            encode::CRITIC_EXTRA
        ),
        (288, 1382, 160, 128, 274)
    );
    assert_eq!(encode::CRITIC_FEATURES, 1382 + 274);

    // Section 3: the token fields and the event vocabulary.
    let fields = table_with(&tables, &["Field", "Values"]);
    assert_eq!(fields.len(), encode::TOKEN_FIELDS);
    let sizes: Vec<usize> = fields
        .iter()
        .map(|row| numbers(&row[1]).into_iter().max().unwrap() + 1)
        .collect();
    assert_eq!(sizes, encode::TOKEN_FIELD_SIZES);
    assert_eq!(sizes[0], encode::N_EVENTS);
    assert_eq!(sizes[4], encode::MAX_TOKENS);
    assert_eq!(usize::try_from(encode::NO_SEAT).unwrap(), sizes[1] - 1);

    let events: Vec<(String, usize, usize)> = table_with(&tables, &["Id", "Count", "Name"])
        .iter()
        .map(|row| {
            let (first, count) = span(&row[0]);
            assert_eq!(count, row[1].parse::<usize>().unwrap(), "{}", row[2]);
            (group_name(&row[2]), first, count)
        })
        .collect();
    let exported: Vec<(String, usize, usize)> = encode::EVENT_GROUPS
        .iter()
        .map(|&(name, first, count)| (name.to_string(), first, count))
        .collect();
    assert_eq!(exported, events);
    assert_eq!(events.iter().map(|e| e.2).sum::<usize>(), encode::N_EVENTS);

    // Section 4: every state block with its offset and width, and the total.
    let mut blocks = Vec::new();
    let mut total = None;
    for row in table_with(&tables, &["Block", "Width", "Offset"]) {
        let name = row[0].replace('*', "");
        let head = name.split_whitespace().next().unwrap().to_string();
        let is_block = head.len() >= 2
            && head.chars().next().unwrap().is_ascii_uppercase()
            && head[1..].chars().all(|c| c.is_ascii_digit());
        if is_block {
            let (offset, count) = span(&row[2]);
            let width: usize = row[1].parse().unwrap();
            assert_eq!(count, width, "{name}");
            blocks.push((head, offset, width));
        } else if name.starts_with("Total") {
            total = Some(numbers(&row[1])[0]);
        }
    }
    let exported: Vec<(String, usize, usize)> = encode::STATE_BLOCKS
        .iter()
        .map(|&(name, offset, width)| (name.to_string(), offset, width))
        .collect();
    assert_eq!(exported, blocks);
    assert_eq!(total, Some(encode::STATE_FEATURES));
    // No gap and no overlap, from 0 to the end.
    let mut next = 0;
    for (name, offset, width) in &blocks {
        assert_eq!(*offset, next, "{name}");
        next = offset + width;
    }
    assert_eq!(next, encode::STATE_FEATURES);

    // Section 5: the action slots.
    let actions: Vec<(String, usize, usize)> = table_with(&tables, &["Slot", "Count", "Name"])
        .iter()
        .map(|row| {
            let (first, count) = span(&row[0]);
            assert_eq!(count, row[1].parse::<usize>().unwrap(), "{}", row[2]);
            (group_name(&row[2]), first, count)
        })
        .collect();
    let exported: Vec<(String, usize, usize)> = encode::ACTION_GROUPS
        .iter()
        .map(|&(name, first, count)| (name.to_string(), first, count))
        .collect();
    assert_eq!(exported, actions);
    assert_eq!(
        actions.iter().map(|a| a.2).sum::<usize>(),
        encode::N_ACTIONS
    );
    assert_eq!(encode::N_ACTIONS, tarok_engine::Action::SLOTS);

    // Section 6: the critic's extra features.
    let extra: Vec<usize> = table_with(&tables, &["Width", "Content"])
        .iter()
        .map(|row| row[0].parse().unwrap())
        .collect();
    let exported: Vec<usize> = encode::CRITIC_EXTRA_BLOCKS
        .iter()
        .map(|&(_, width)| width)
        .collect();
    assert_eq!(exported, extra);
    assert_eq!(extra.iter().sum::<usize>(), encode::CRITIC_EXTRA);

    // Section 8: the reward scale and the ten score parts, in order.
    assert!(text.contains("/ 50."));
    assert_eq!(encode::REWARD_SCALE, 50.0);
    let parts = table_with(&tables, &["Index", "Part"]);
    for (index, row) in parts.iter().enumerate() {
        assert_eq!(row[0].parse::<usize>().unwrap(), index);
    }
    let names: Vec<&str> = parts.iter().map(|row| row[1].as_str()).collect();
    assert_eq!(names, encode::SCORE_PARTS);
    assert_eq!(encode::N_SCORE_PARTS, 10);
}

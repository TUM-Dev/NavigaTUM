//! Minimal Rust parser/writer for `data/processors/areatree/config.areatree`.
//!
//! Mirrors `data/processors/areatree/process.py` for the two operations that the addition path
//! needs:
//!   - **Index** every line by `id` with its `kind`, `parents` chain, and indent depth (used by
//!     [`super::validation::RepoSnapshot`]).
//!   - **Insert** a new line under a given parent at the right indent, sorted alphabetically
//!     among existing siblings.
//!
//! The full areatree DSL (warnings about embedded short names, pattern matching for
//! `_TUMONLINE_*_RE`, …) is intentionally NOT re-implemented here — we only need to know which
//! IDs exist and where to insert.
use std::str::FromStr as _;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, strum::IntoStaticStr, strum::EnumString, strum::Display,
)]
#[strum(serialize_all = "snake_case")]
pub enum AreatreeKind {
    Root,
    Site,
    Campus,
    Area,
    Building,
    JoinedBuilding,
}

#[derive(Debug, Clone)]
pub struct AreatreeNode {
    pub id: String,
    pub kind: AreatreeKind,
    /// All `b_prefix` entries for this row. Empty if the node has no building prefix.
    pub b_prefixes: Vec<String>,
    pub visible_id: Option<String>,
    pub parents: Vec<String>,
    /// Indent level (in pairs of spaces).
    pub indent_level: usize,
    /// 1-based line number in the source file (for error messages and stable sort tiebreaker).
    pub line_no: usize,
}

#[derive(Debug, Clone, Default)]
pub struct AreatreeIndex {
    pub nodes: Vec<AreatreeNode>,
}

impl AreatreeIndex {
    pub fn parse(content: &str) -> anyhow::Result<Self> {
        let mut nodes = Vec::new();
        let mut parent_stack: Vec<String> = Vec::new();
        let mut last_id: Option<String> = None;

        for (idx, raw_line) in content.lines().enumerate() {
            let stripped = strip_comment_and_trailing_ws(raw_line);
            if stripped.is_empty() {
                continue;
            }
            let indent_spaces = raw_line.len() - raw_line.trim_start_matches(' ').len();
            if indent_spaces % 2 != 0 {
                anyhow::bail!("line {} indent not multiple of 2: '{raw_line}'", idx + 1);
            }
            let indent_level = indent_spaces / 2;
            if indent_level > parent_stack.len() {
                if let Some(id) = last_id.take() {
                    parent_stack.push(id);
                }
            } else if indent_level < parent_stack.len() {
                parent_stack.truncate(indent_level);
            }

            let line_content = stripped.trim_start_matches(' ');
            let parsed = parse_line(line_content)
                .map_err(|e| anyhow::anyhow!("line {}: {e}: '{raw_line}'", idx + 1))?;

            last_id = Some(parsed.id.clone());
            nodes.push(AreatreeNode {
                id: parsed.id,
                kind: parsed.kind,
                b_prefixes: parsed.b_prefixes,
                visible_id: parsed.visible_id,
                parents: parent_stack.clone(),
                indent_level,
                line_no: idx + 1,
            });
        }
        Ok(Self { nodes })
    }

    pub fn iter(&self) -> impl Iterator<Item = &AreatreeNode> {
        self.nodes.iter()
    }

    pub fn find(&self, id: &str) -> Option<&AreatreeNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn contains_id(&self, id: &str) -> bool {
        self.nodes.iter().any(|n| n.id == id)
    }

    pub fn contains_b_prefix(&self, prefix: &str) -> bool {
        self.nodes
            .iter()
            .any(|n| n.b_prefixes.iter().any(|p| p == prefix))
    }

    pub fn contains_visible_id(&self, vid: &str) -> bool {
        self.nodes
            .iter()
            .any(|n| n.visible_id.as_deref() == Some(vid))
    }
}

#[derive(Debug)]
struct ParsedLine {
    id: String,
    kind: AreatreeKind,
    b_prefixes: Vec<String>,
    visible_id: Option<String>,
}

fn strip_comment_and_trailing_ws(line: &str) -> &str {
    let no_comment = match line.find('#') {
        Some(i) => &line[..i],
        None => line,
    };
    no_comment.trim_end()
}

fn parse_line(content: &str) -> anyhow::Result<ParsedLine> {
    let parts: Vec<&str> = content.split(':').collect();
    if parts.len() != 3 {
        anyhow::bail!("expected 3 ':'-separated parts");
    }
    let building_ids = parts[0].trim();
    let _names = parts[1].trim();
    let internal_id_raw = parts[2].trim();

    // Building prefix(es). Strip leading "-" data-quality marker.
    let bp_part = building_ids.trim_start_matches('-');
    let b_prefixes: Vec<String> = if bp_part.is_empty() {
        Vec::new()
    } else if bp_part.contains(',') {
        bp_part.split(',').map(|s| s.trim().to_string()).collect()
    } else {
        vec![bp_part.to_string()]
    };

    // Internal id parsing: <id>[,<visible_id>][[<type>]]
    let (internal_no_type, kind_override) = if let Some(open) = internal_id_raw.find('[') {
        let inner = internal_id_raw
            .get(open + 1..)
            .and_then(|s| s.strip_suffix(']'))
            .ok_or_else(|| anyhow::anyhow!("malformed [type] in '{internal_id_raw}'"))?;
        if inner.contains(',') {
            anyhow::bail!("type comes after visible_ids: '{internal_id_raw}'");
        }
        (
            internal_id_raw[..open].trim_end(),
            Some(
                AreatreeKind::from_str(inner.trim())
                    .map_err(|_| anyhow::anyhow!("unknown areatree node type `{}`", inner.trim()))?,
            ),
        )
    } else {
        (internal_id_raw, None)
    };

    let (id, visible_id) = if internal_no_type.contains(',') {
        let mut split = internal_no_type.split(',');
        let id = split
            .next()
            .ok_or_else(|| anyhow::anyhow!("empty id"))?
            .trim()
            .to_string();
        let vid = split
            .next()
            .ok_or_else(|| anyhow::anyhow!("missing visible_id"))?
            .trim()
            .to_string();
        if split.next().is_some() {
            anyhow::bail!("more than two ids in '{internal_no_type}'");
        }
        (id, Some(vid))
    } else if !internal_no_type.is_empty() {
        (internal_no_type.to_string(), None)
    } else if b_prefixes.len() == 1 {
        (b_prefixes[0].clone(), None)
    } else {
        anyhow::bail!("no id provided");
    };

    let kind = match kind_override {
        Some(k) => k,
        None => {
            if b_prefixes.len() == 1 && b_prefixes[0] == id {
                AreatreeKind::Building
            } else {
                AreatreeKind::Area
            }
        }
    };

    Ok(ParsedLine {
        id,
        kind,
        b_prefixes,
        visible_id,
    })
}

/// Reconstruct an areatree line for a new node. Mirrors `_format_line` in process.py.
pub fn format_line(
    b_prefixes: &[String],
    name: &str,
    short_name: Option<&str>,
    id: &str,
    visible_id: Option<&str>,
    kind: AreatreeKind,
) -> String {
    let bp_str = b_prefixes.join(",");
    let name_str = match short_name {
        Some(sn) => format!("{name}|{sn}"),
        None => name.to_string(),
    };
    let id_str = match visible_id {
        Some(v) => format!("{id},{v}"),
        None => id.to_string(),
    };

    // The type bracket is only emitted when the kind would NOT be inferred from the line:
    //   - building when single prefix == id
    //   - area otherwise
    let inferred_kind = if b_prefixes.len() == 1 && b_prefixes[0] == id {
        AreatreeKind::Building
    } else {
        AreatreeKind::Area
    };
    let type_str = if kind == inferred_kind {
        String::new()
    } else {
        format!("[{kind}]")
    };

    format!("{bp_str}:{name_str}:{id_str}{type_str}")
}

/// Insert `new_line_content` (no trailing newline, no leading indent) under `parent_id`.
///
/// Inserts among existing direct children of `parent_id`, alphabetically sorted by their primary
/// id. If no parent or no existing children, appends after the parent's line block.
pub fn insert_under(
    file_content: &str,
    parent_id: &str,
    new_id: &str,
    new_line_content: &str,
) -> anyhow::Result<String> {
    let index = AreatreeIndex::parse(file_content)?;
    let parent = index
        .find(parent_id)
        .ok_or_else(|| anyhow::anyhow!("parent `{parent_id}` not found"))?;
    let new_indent_level = parent.indent_level + 1;
    let new_indent = "  ".repeat(new_indent_level);
    let new_line = format!("{new_indent}{new_line_content}");

    // Direct children of the parent (immediately deeper indent, parent is in their parents chain).
    let direct_children: Vec<&AreatreeNode> = index
        .iter()
        .filter(|n| {
            n.indent_level == new_indent_level
                && n.parents.last().map(String::as_str) == Some(parent_id)
        })
        .collect();

    // Find the source line we want to insert BEFORE. We walk siblings in source order; pick the
    // first sibling whose id is alphabetically > new_id. If none, insert after the LAST line that
    // belongs to the parent's subtree.
    let lines: Vec<&str> = file_content.lines().collect();
    let insert_before_lineno: usize =
        if let Some(next_sibling) = direct_children.iter().find(|c| c.id.as_str() > new_id) {
            next_sibling.line_no
        } else if let Some(last_in_subtree_lineno) = last_subtree_lineno(&index, parent_id) {
            last_in_subtree_lineno + 1
        } else {
            parent.line_no + 1
        };

    let mut out = String::with_capacity(file_content.len() + new_line.len() + 1);
    let trailing_newline = file_content.ends_with('\n');
    let total = lines.len();

    for (i, line) in lines.iter().enumerate() {
        let lineno = i + 1; // 1-based
        if lineno == insert_before_lineno {
            out.push_str(&new_line);
            out.push('\n');
        }
        out.push_str(line);
        if i + 1 < total || trailing_newline {
            out.push('\n');
        }
    }

    if insert_before_lineno > total {
        if !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(&new_line);
        if trailing_newline {
            out.push('\n');
        }
    }

    Ok(out)
}

fn last_subtree_lineno(index: &AreatreeIndex, parent_id: &str) -> Option<usize> {
    index
        .iter()
        .filter(|n| n.parents.iter().any(|p| p == parent_id) || n.id == parent_id)
        .map(|n| n.line_no)
        .max()
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic, clippy::panic_in_result_fn)]
mod tests {
    use insta::assert_snapshot;
    use rstest::rstest;

    use super::*;

    const SAMPLE: &str = "\
:Standorte:root[root]
  0:Stammgelände:stammgelaende[campus]
    01:Nordgelände:nordgelaende
      0101:Hörsäle (U-Trakt)|N1:0101,n1
      0102:Hochvolthaus|N2:0102,n2
    02:Südgelände:suedgelaende
      0201:Gabelsbergerstr. 43|S1:0201,s1
";

    #[test]
    fn parses_basic_structure() {
        let idx = AreatreeIndex::parse(SAMPLE).unwrap();
        assert_eq!(idx.nodes.len(), 7);
        let root = idx.find("root").unwrap();
        assert_eq!(root.kind, AreatreeKind::Root);
        let stamm = idx.find("stammgelaende").unwrap();
        assert_eq!(stamm.kind, AreatreeKind::Campus);
        assert_eq!(stamm.parents, vec!["root".to_string()]);
        let n1 = idx.find("0101").unwrap();
        assert_eq!(n1.kind, AreatreeKind::Building);
        assert_eq!(n1.b_prefixes, vec!["0101".to_string()]);
        assert_eq!(n1.visible_id.as_deref(), Some("n1"));
        assert_eq!(
            n1.parents,
            vec![
                "root".to_string(),
                "stammgelaende".to_string(),
                "nordgelaende".to_string()
            ]
        );
    }

    #[test]
    fn skips_comment_and_blank_lines() {
        let content = "\
# header
:Standorte:root[root]

  0:Stammgelände:stammgelaende[campus]  # inline comment
    01:Nordgelände:nordgelaende
";
        let idx = AreatreeIndex::parse(content).unwrap();
        assert_eq!(idx.nodes.len(), 3);
    }

    #[test]
    fn insert_under_alphabetical_middle() {
        let result = insert_under(SAMPLE, "nordgelaende", "0103", "0103:NewBldg:0103,n3").unwrap();
        assert_snapshot!(result, @r"
        :Standorte:root[root]
          0:Stammgelände:stammgelaende[campus]
            01:Nordgelände:nordgelaende
              0101:Hörsäle (U-Trakt)|N1:0101,n1
              0102:Hochvolthaus|N2:0102,n2
              0103:NewBldg:0103,n3
            02:Südgelände:suedgelaende
              0201:Gabelsbergerstr. 43|S1:0201,s1
        ");
    }

    #[test]
    fn insert_under_alphabetical_first() {
        let result = insert_under(SAMPLE, "nordgelaende", "0100", "0100:Foo:0100").unwrap();
        assert_snapshot!(result, @r"
        :Standorte:root[root]
          0:Stammgelände:stammgelaende[campus]
            01:Nordgelände:nordgelaende
              0100:Foo:0100
              0101:Hörsäle (U-Trakt)|N1:0101,n1
              0102:Hochvolthaus|N2:0102,n2
            02:Südgelände:suedgelaende
              0201:Gabelsbergerstr. 43|S1:0201,s1
        ");
    }

    #[test]
    fn insert_under_alphabetical_last() {
        let result = insert_under(SAMPLE, "nordgelaende", "9999", "9999:Foo:9999").unwrap();
        assert_snapshot!(result, @r"
        :Standorte:root[root]
          0:Stammgelände:stammgelaende[campus]
            01:Nordgelände:nordgelaende
              0101:Hörsäle (U-Trakt)|N1:0101,n1
              0102:Hochvolthaus|N2:0102,n2
              9999:Foo:9999
            02:Südgelände:suedgelaende
              0201:Gabelsbergerstr. 43|S1:0201,s1
        ");
    }

    #[test]
    fn insert_under_first_child() {
        let content = "\
:Standorte:root[root]
  0:Stammgelände:stammgelaende[campus]
";
        let result = insert_under(content, "stammgelaende", "01", "01:NewArea:newarea").unwrap();
        assert_snapshot!(result, @r"
        :Standorte:root[root]
          0:Stammgelände:stammgelaende[campus]
            01:NewArea:newarea
        ");
    }

    #[rstest]
    #[case::inferred_building(
        &["5117"], "New Bldg", None, "5117", None, AreatreeKind::Building,
        "5117:New Bldg:5117"
    )]
    #[case::short_and_visible(
        &["5117"], "New Bldg", Some("NB"), "5117", Some("nb"), AreatreeKind::Building,
        "5117:New Bldg|NB:5117,nb"
    )]
    #[case::explicit_joined_building(
        &["15", "17"], "MRI joined", None, "1517", None, AreatreeKind::JoinedBuilding,
        "15,17:MRI joined:1517[joined_building]"
    )]
    fn format_line_cases(
        #[case] prefixes: &[&str],
        #[case] name: &str,
        #[case] short: Option<&str>,
        #[case] id: &str,
        #[case] visible: Option<&str>,
        #[case] kind: AreatreeKind,
        #[case] expected: &str,
    ) {
        let prefixes: Vec<String> = prefixes.iter().map(|p| (*p).to_string()).collect();
        assert_eq!(
            format_line(&prefixes, name, short, id, visible, kind),
            expected
        );
    }

    #[test]
    fn parse_real_areatree_snippet() {
        // The Cargo manifest sets the working dir for `cargo test` to the server crate. Resolve
        // upward to the project root (containing `data/`) — fall back gracefully if running in
        // a non-checkout environment.
        let candidate_paths = [
            "../data/processors/areatree/config.areatree",
            "data/processors/areatree/config.areatree",
        ];
        let content = candidate_paths
            .iter()
            .find_map(|p| std::fs::read_to_string(p).ok())
            .expect("config.areatree must be reachable for this integration check");
        let idx = AreatreeIndex::parse(&content).expect("real areatree must parse");
        assert!(
            idx.find("root").is_some(),
            "real areatree must define `root`"
        );
        assert!(
            idx.nodes.len() > 100,
            "expected the real areatree to be large"
        );
    }
}

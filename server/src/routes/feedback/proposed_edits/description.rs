use std::collections::{BTreeMap, HashMap};
use std::fmt::Write as _;
use std::path::Path;

use tracing::error;

use super::AppliableEdit;
use super::addition::Addition;
use crate::limited::hash_map::LimitedHashMap;

#[derive(Default)]
pub struct Description {
    pub title: String,
    pub body: String,
}

impl Description {
    pub fn add_context(&mut self, additional_context: &str) {
        if !additional_context.is_empty() {
            writeln!(self.body, "## Additional context:\n> {additional_context}")
                .expect("writing to a String is infallible");
        }
    }
    pub fn apply_set<T: AppliableEdit>(
        &mut self,
        category_name: &'static str,
        set: HashMap<String, T>,
        base_dir: &Path,
        branch: &str,
    ) -> anyhow::Result<()> {
        if !set.is_empty() {
            let edits = if set.len() == 1 { "edit" } else { "edits" };
            let amount = set.len();
            if self.title.is_empty() {
                self.title = format!("{amount} {category_name} {edits}");
            } else {
                write!(self.title, " and {amount} {category_name} {edits}")?;
            }

            writeln!(
                self.body,
                "\nThe following {category_name} edits were made:"
            )?;

            self.body += "| entry | edit |\n";
            self.body += "| ---   | ---  |\n";
            for (key, value) in set {
                let result = value
                    .apply(&key, base_dir, branch)
                    .inspect_err(|e| error!(error=?e, %key, %category_name, "apply failed"))?;
                writeln!(
                    self.body,
                    "| [`{key}`](https://nav.tum.de/view/{key}) | {result} |"
                )?;
            }
        }
        Ok(())
    }

    pub fn apply_additions(
        &mut self,
        additions: &LimitedHashMap<String, Addition>,
        base_dir: &Path,
        branch: &str,
    ) -> anyhow::Result<()> {
        if additions.0.is_empty() {
            return Ok(());
        }
        let mut by_kind: BTreeMap<&'static str, Vec<(&str, &Addition)>> = BTreeMap::new();
        for (k, a) in &additions.0 {
            by_kind.entry(a.kind_label()).or_default().push((k, a));
        }
        for (kind, mut entries) in by_kind {
            entries.sort_by_key(|(k, _)| k.to_string());
            let plural = if entries.len() == 1 {
                "addition"
            } else {
                "additions"
            };
            let n = entries.len();
            if self.title.is_empty() {
                self.title = format!("{n} {kind} {plural}");
            } else {
                write!(self.title, " and {n} {kind} {plural}")?;
            }
            writeln!(self.body, "\nThe following {kind} additions were made:")?;

            // Building summaries are multi-line GeoJSON, which doesn't fit in a table cell.
            let use_blocks = kind == "building";
            if use_blocks {
                for (key, addition) in &entries {
                    let result = addition
                        .apply(key, base_dir, branch)
                        .inspect_err(|e| error!(error=?e, %key, %kind, "addition apply failed"))?;
                    let indented = result
                        .lines()
                        .map(|line| format!("    {line}"))
                        .collect::<Vec<_>>()
                        .join("\n");
                    writeln!(
                        self.body,
                        "- [`{key}`](https://nav.tum.de/view/{key}):\n\n{indented}"
                    )?;
                }
            } else {
                self.body += "| entry | addition |\n";
                self.body += "| ---   | ---      |\n";
                for (key, addition) in &entries {
                    let result = addition
                        .apply(key, base_dir, branch)
                        .inspect_err(|e| error!(error=?e, %key, %kind, "addition apply failed"))?;
                    writeln!(
                        self.body,
                        "| [`{key}`](https://nav.tum.de/view/{key}) | {result} |"
                    )?;
                }
            }
        }
        Ok(())
    }

    pub fn apply_set_as_blocks<T: AppliableEdit>(
        &mut self,
        category_name: &'static str,
        set: HashMap<String, T>,
        base_dir: &Path,
        branch: &str,
    ) -> anyhow::Result<()> {
        if !set.is_empty() {
            let edits = if set.len() == 1 { "edit" } else { "edits" };
            let amount = set.len();
            if self.title.is_empty() {
                self.title = format!("{amount} {category_name} {edits}");
            } else {
                write!(self.title, " and {amount} {category_name} {edits}")?;
            }

            writeln!(
                self.body,
                "\nThe following {category_name} edits were made:"
            )?;

            for (key, value) in set {
                let result = value
                    .apply(&key, base_dir, branch)
                    .inspect_err(|e| error!(error=?e, %key, %category_name, "apply failed"))?;
                let indented_result = result
                    .lines()
                    .map(|line| format!("    {line}"))
                    .collect::<Vec<_>>()
                    .join("\n");
                writeln!(
                    self.body,
                    "- [`{key}`](https://nav.tum.de/view/{key}):\n\n{indented_result}"
                )?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::panic,
        clippy::panic_in_result_fn,
        clippy::zero_sized_map_values,
        reason = "tests assert via panic/unwrap and use zero-sized test edits"
    )]
    use std::collections::HashMap;
    use std::path::Path;

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_description_context() {
        let mut description = Description {
            title: "title".to_string(),
            body: "body\n".to_string(),
        };
        description.add_context("context");
        description.add_context(""); // should be a noop
        assert_eq!(description.title, "title");
        assert_eq!(
            description.body,
            "body\n## Additional context:\n> context\n"
        );
    }

    #[derive(Default)]
    struct TestEdit;
    impl AppliableEdit for TestEdit {
        fn apply(&self, _key: &str, _base_dir: &Path, _branch: &str) -> anyhow::Result<String> {
            Ok("applied_value".to_string())
        }
    }

    #[test]
    fn test_apply_set_empty() {
        let mut description = Description::default();
        let set: HashMap<String, TestEdit> = HashMap::default();
        description
            .apply_set("category", set, Path::new(""), "none")
            .unwrap();
        assert_eq!(description.title, "");
        assert_eq!(description.body, "");
    }

    #[test]
    fn test_apply_set() {
        let mut description = Description::default();
        let set = HashMap::from([("key".to_string(), TestEdit)]);
        description
            .apply_set("category", set, Path::new(""), "none")
            .unwrap();
        assert_eq!(description.title, "1 category edit");
        assert_eq!(
            description.body,
            "\nThe following category edits were made:\n| entry | edit |\n| ---   | ---  |\n| [`key`](https://nav.tum.de/view/key) | applied_value |\n"
        );
    }

    #[test]
    fn test_apply_set_as_blocks_empty() {
        let mut description = Description::default();
        let set: HashMap<String, TestEdit> = HashMap::default();
        description
            .apply_set_as_blocks("coordinate", set, Path::new(""), "none")
            .unwrap();
        assert_eq!(description.title, "");
        assert_eq!(description.body, "");
    }

    #[test]
    fn test_apply_set_as_blocks() {
        let mut description = Description::default();
        let set = HashMap::from([("key".to_string(), TestEdit)]);
        description
            .apply_set_as_blocks("coordinate", set, Path::new(""), "none")
            .unwrap();
        assert_eq!(description.title, "1 coordinate edit");
        // A blank line after the list-item colon and 4-space indentation ensure the fenced block
        // is rendered as content of the list item in GitHub-flavored Markdown.
        assert_eq!(
            description.body,
            "\nThe following coordinate edits were made:\n- [`key`](https://nav.tum.de/view/key):\n\n    applied_value\n"
        );
    }

    #[test]
    fn test_apply_set_as_blocks_does_not_use_table() {
        let mut description = Description::default();
        let set = HashMap::from([("key".to_string(), TestEdit)]);
        description
            .apply_set_as_blocks("coordinate", set, Path::new(""), "none")
            .unwrap();
        // Block output must not contain table syntax.
        assert!(!description.body.contains("| entry |"));
        assert!(!description.body.contains("| ---"));
    }
}

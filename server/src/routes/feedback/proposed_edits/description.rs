use std::collections::HashMap;
use std::path::Path;

use super::AppliableEdit;

#[derive(Default)]
pub struct Description {
    pub title: String,
    pub body: String,
}

impl Description {
    pub fn add_context(&mut self, additional_context: &str) {
        if !additional_context.is_empty() {
            self.body += &format!("## Additional context:\n> {additional_context}\n");
        }
    }
    pub fn appply_set<T: AppliableEdit>(
        &mut self,
        category_name: &'static str,
        set: HashMap<String, T>,
        base_dir: &Path,
        branch: &str,
    ) {
        if !set.is_empty() {
            let edits = if set.len() == 1 { "edit" } else { "edits" };
            if self.title.is_empty() {
                self.title = format!("{amount} {category_name} {edits}", amount = set.len());
            } else {
                self.title += &format!(" and {amount} {category_name} {edits}", amount = set.len());
            }

            self.body += &format!("\nThe following {category_name} edits were made:\n");

            self.body += "| entry | edit |\n";
            self.body += "| ---   | ---  |\n";
            for (key, value) in set {
                let result = value.apply(&key, base_dir, branch);
                self.body += &format!("| [`{key}`](https://nav.tum.de/view/{key}) | {result} |\n");
            }
        }
    }
}

#[cfg(test)]
mod tests {
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
        fn apply(&self, _key: &str, _base_dir: &Path) -> String {
            "applied_value".to_string()
        }
    }

    #[test]
    fn test_apply_set_empty() {
        let mut description = Description::default();
        let set: HashMap<String, TestEdit> = HashMap::default();
        description.appply_set("category", set, Path::new(""), "none");
        assert_eq!(description.title, "");
        assert_eq!(description.body, "");
    }

    #[test]
    fn test_apply_set() {
        let mut description = Description::default();
        let set = HashMap::from([("key".to_string(), TestEdit)]);
        description.appply_set("category", set, Path::new(""), "none");
        assert_eq!(description.title, "1 category edit");
        assert_eq!(
            description.body,
            "\nThe following category edits were made:\n| entry | edit |\n| ---   | ---  |\n| [`key`](https://nav.tum.de/view/key) | applied_value |\n"
        );
    }
}

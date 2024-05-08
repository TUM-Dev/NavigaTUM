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
            self.body += &format!("Additional context: {additional_context}\n");
        }
    }
    pub fn appply_set<T: AppliableEdit>(
        &mut self,
        category_name: &'static str,
        set: HashMap<String, T>,
        base_dir: &Path,
    ) {
        if !set.is_empty() {
            self.title = format!("{amount} {category_name} edits", amount = set.len());

            self.body += &format!("The following {category_name} edits were made:\n");

            self.body += "| entry | edit | \n";
            self.body += "| --- | --- | \n";
            for (key, value) in set {
                let result = value.apply(&key, base_dir);
                self.body += &format!("| [`{key}`](https://nav.tum.de/view/{key}) | {result} |\n");
            }
        }
    }
}

#[cfg(test)]
mod test_discription {
    use std::collections::HashMap;
    use std::path::Path;

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
        assert_eq!(description.body, "body\nAdditional context: context\n");
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
        description.appply_set("category", set, Path::new(""));
        assert_eq!(description.title, "");
        assert_eq!(description.body, "");
    }

    #[test]
    fn test_apply_set() {
        let mut description = Description::default();
        let set = HashMap::from([("key".to_string(), TestEdit)]);
        description.appply_set("category", set, Path::new(""));
        assert_eq!(description.title, "1 category edits");
        assert_eq!(description.body, "The following category edits were made:\n| entry | edit | \n| --- | --- | \n| [`key`](https://nav.tum.de/view/key) | applied_value |\n");
    }
}

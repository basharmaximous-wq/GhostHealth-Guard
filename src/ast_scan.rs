use syn::{visit::Visit, File, ItemFn, Attribute};
use std::fs;
use walkdir::WalkDir;
use crate::models::Issue;

pub fn scan_rust_project(path: &str) -> Vec<Issue> {
    let mut issues = vec![];

    for entry in WalkDir::new(path).into_iter().filter_map(Result::ok) {
        if entry.path().extension().map(|e| e == "rs").unwrap_or(false) {
            let content = fs::read_to_string(entry.path()).unwrap_or_default();
            if let Ok(parsed) = syn::parse_file(&content) {
                let mut visitor = RustVisitor::default();
                visitor.visit_file(&parsed);
                issues.extend(visitor.issues);
            }
        }
    }

    issues
}

#[derive(Default)]
struct RustVisitor {
    issues: Vec<Issue>,
}

impl<'ast> Visit<'ast> for RustVisitor {

    fn visit_item_fn(&mut self, node: &'ast ItemFn) {

        // Detect unsafe blocks
        if node.sig.unsafety.is_some() {
            self.issues.push(Issue {
                category: "UNSAFE_FUNCTION".into(),
                severity: "MEDIUM".into(),
                message: format!("Unsafe function: {}", node.sig.ident),
            });
        }

        // Detect #[Sensitive] attribute usage
        for attr in &node.attrs {
            if is_sensitive(attr) {
                self.issues.push(Issue {
                    category: "SENSITIVE_FUNCTION".into(),
                    severity: "HIGH".into(),
                    message: format!("Sensitive function exposed: {}", node.sig.ident),
                });
            }
        }

        syn::visit::visit_item_fn(self, node);
    }
}

fn is_sensitive(attr: &Attribute) -> bool {
    attr.path().is_ident("Sensitive")
}

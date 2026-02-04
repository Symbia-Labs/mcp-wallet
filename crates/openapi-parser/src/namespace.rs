//! Namespace tree for fast operation lookup

use crate::types::{ApiOperation, HttpMethod};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A tree structure for fast operation lookup by namespace
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NamespaceTree {
    /// Child nodes
    pub children: HashMap<String, NamespaceTree>,
    /// Operation at this node (if any)
    pub operation: Option<OperationRef>,
}

/// Reference to an operation in the namespace tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationRef {
    /// Operation ID
    pub operation_id: String,
    /// HTTP method
    pub method: HttpMethod,
    /// URL path
    pub path: String,
    /// Index in the operations list
    pub index: usize,
}

impl NamespaceTree {
    /// Create a new empty namespace tree
    pub fn new() -> Self {
        Self::default()
    }

    /// Build a namespace tree from a list of operations
    pub fn build(operations: &[ApiOperation]) -> Self {
        let mut tree = Self::new();

        for (index, op) in operations.iter().enumerate() {
            tree.insert(
                &op.normalized_id,
                OperationRef {
                    operation_id: op.operation_id.clone(),
                    method: op.method,
                    path: op.path.clone(),
                    index,
                },
            );
        }

        tree
    }

    /// Insert an operation into the tree
    pub fn insert(&mut self, path: &str, operation: OperationRef) {
        let parts: Vec<&str> = path.split('.').collect();
        self.insert_parts(&parts, operation);
    }

    fn insert_parts(&mut self, parts: &[&str], operation: OperationRef) {
        if parts.is_empty() {
            self.operation = Some(operation);
            return;
        }

        let child = self
            .children
            .entry(parts[0].to_string())
            .or_default();

        child.insert_parts(&parts[1..], operation);
    }

    /// Look up an operation by its namespace path
    pub fn lookup(&self, path: &str) -> Option<&OperationRef> {
        let parts: Vec<&str> = path.split('.').collect();
        self.lookup_parts(&parts)
    }

    fn lookup_parts(&self, parts: &[&str]) -> Option<&OperationRef> {
        if parts.is_empty() {
            return self.operation.as_ref();
        }

        self.children
            .get(parts[0])
            .and_then(|child| child.lookup_parts(&parts[1..]))
    }

    /// Get all operations under a namespace prefix
    pub fn list(&self, prefix: &str) -> Vec<&OperationRef> {
        if prefix.is_empty() {
            return self.collect_all();
        }

        let parts: Vec<&str> = prefix.split('.').collect();
        self.list_parts(&parts)
    }

    fn list_parts(&self, parts: &[&str]) -> Vec<&OperationRef> {
        if parts.is_empty() {
            return self.collect_all();
        }

        self.children
            .get(parts[0])
            .map(|child| child.list_parts(&parts[1..]))
            .unwrap_or_default()
    }

    /// Collect all operations in this subtree
    fn collect_all(&self) -> Vec<&OperationRef> {
        let mut ops = Vec::new();

        if let Some(ref op) = self.operation {
            ops.push(op);
        }

        for child in self.children.values() {
            ops.extend(child.collect_all());
        }

        ops
    }

    /// Get all namespace paths
    pub fn paths(&self) -> Vec<String> {
        self.collect_paths("")
    }

    fn collect_paths(&self, prefix: &str) -> Vec<String> {
        let mut paths = Vec::new();

        if self.operation.is_some() {
            paths.push(prefix.to_string());
        }

        for (name, child) in &self.children {
            let child_prefix = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{}.{}", prefix, name)
            };
            paths.extend(child.collect_paths(&child_prefix));
        }

        paths
    }

    /// Get child namespaces at a given prefix
    pub fn children_at(&self, prefix: &str) -> Vec<String> {
        if prefix.is_empty() {
            return self.children.keys().cloned().collect();
        }

        let parts: Vec<&str> = prefix.split('.').collect();
        self.children_at_parts(&parts)
    }

    fn children_at_parts(&self, parts: &[&str]) -> Vec<String> {
        if parts.is_empty() {
            return self.children.keys().cloned().collect();
        }

        self.children
            .get(parts[0])
            .map(|child| child.children_at_parts(&parts[1..]))
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_tree() -> NamespaceTree {
        let operations = vec![
            ApiOperation {
                operation_id: "createCustomer".to_string(),
                normalized_id: "customers.create".to_string(),
                method: HttpMethod::Post,
                path: "/v1/customers".to_string(),
                summary: None,
                description: None,
                tags: vec![],
                deprecated: false,
                parameters: vec![],
                request_body: None,
                responses: vec![],
                security: vec![],
            },
            ApiOperation {
                operation_id: "getCustomer".to_string(),
                normalized_id: "customers.get".to_string(),
                method: HttpMethod::Get,
                path: "/v1/customers/{id}".to_string(),
                summary: None,
                description: None,
                tags: vec![],
                deprecated: false,
                parameters: vec![],
                request_body: None,
                responses: vec![],
                security: vec![],
            },
            ApiOperation {
                operation_id: "listCustomers".to_string(),
                normalized_id: "customers.list".to_string(),
                method: HttpMethod::Get,
                path: "/v1/customers".to_string(),
                summary: None,
                description: None,
                tags: vec![],
                deprecated: false,
                parameters: vec![],
                request_body: None,
                responses: vec![],
                security: vec![],
            },
        ];

        NamespaceTree::build(&operations)
    }

    #[test]
    fn test_lookup() {
        let tree = test_tree();

        let op = tree.lookup("customers.create").unwrap();
        assert_eq!(op.operation_id, "createCustomer");
        assert_eq!(op.method, HttpMethod::Post);
    }

    #[test]
    fn test_lookup_not_found() {
        let tree = test_tree();
        assert!(tree.lookup("orders.create").is_none());
    }

    #[test]
    fn test_list() {
        let tree = test_tree();

        let ops = tree.list("customers");
        assert_eq!(ops.len(), 3);
    }

    #[test]
    fn test_paths() {
        let tree = test_tree();

        let paths = tree.paths();
        assert!(paths.contains(&"customers.create".to_string()));
        assert!(paths.contains(&"customers.get".to_string()));
        assert!(paths.contains(&"customers.list".to_string()));
    }

    #[test]
    fn test_children_at() {
        let tree = test_tree();

        let children = tree.children_at("");
        assert!(children.contains(&"customers".to_string()));

        let children = tree.children_at("customers");
        assert!(children.contains(&"create".to_string()));
        assert!(children.contains(&"get".to_string()));
        assert!(children.contains(&"list".to_string()));
    }
}

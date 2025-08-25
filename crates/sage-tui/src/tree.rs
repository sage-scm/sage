/// Node in a tree structure
#[derive(Debug, Clone)]
pub struct TreeNode {
    /// Node name/label
    pub name: String,
    /// Metadata to display
    pub metadata: Vec<NodeMetadata>,
    /// Child nodes
    pub children: Vec<TreeNode>,
}

impl TreeNode {
    /// Create a new tree node
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            metadata: Vec::new(),
            children: Vec::new(),
        }
    }

    /// Add metadata
    pub fn with_metadata(mut self, metadata: NodeMetadata) -> Self {
        self.metadata.push(metadata);
        self
    }

    /// Add a child node
    pub fn with_child(mut self, child: TreeNode) -> Self {
        self.children.push(child);
        self
    }

    /// Add multiple children
    pub fn with_children(mut self, children: Vec<TreeNode>) -> Self {
        self.children.extend(children);
        self
    }

    /// Mark as current
    pub fn current(mut self) -> Self {
        self.metadata.push(NodeMetadata::Current);
        self
    }

    /// Add ahead count
    pub fn ahead(mut self, count: usize) -> Self {
        if count > 0 {
            self.metadata.push(NodeMetadata::Ahead(count));
        }
        self
    }

    /// Add behind count
    pub fn behind(mut self, count: usize) -> Self {
        if count > 0 {
            self.metadata.push(NodeMetadata::Behind(count));
        }
        self
    }

    /// Mark as draft
    pub fn draft(mut self) -> Self {
        self.metadata.push(NodeMetadata::Draft);
        self
    }
}

/// Metadata for tree nodes
#[derive(Debug, Clone)]
pub enum NodeMetadata {
    /// Current node indicator
    Current,
    /// Commits ahead
    Ahead(usize),
    /// Commits behind
    Behind(usize),
    /// Draft/unpublished
    Draft,
    /// Custom text
    Text(String),
}

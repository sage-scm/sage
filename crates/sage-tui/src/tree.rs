#[derive(Debug, Clone)]
pub struct TreeNode {
    pub name: String,
    pub metadata: Vec<NodeMetadata>,
    pub children: Vec<TreeNode>,
}

impl TreeNode {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            metadata: Vec::new(),
            children: Vec::new(),
        }
    }

    pub fn with_metadata(mut self, metadata: NodeMetadata) -> Self {
        self.metadata.push(metadata);
        self
    }

    pub fn with_child(mut self, child: TreeNode) -> Self {
        self.children.push(child);
        self
    }

    pub fn with_children(mut self, children: Vec<TreeNode>) -> Self {
        self.children.extend(children);
        self
    }

    pub fn current(mut self) -> Self {
        self.metadata.push(NodeMetadata::Current);
        self
    }

    pub fn ahead(mut self, count: usize) -> Self {
        if count > 0 {
            self.metadata.push(NodeMetadata::Ahead(count));
        }
        self
    }

    pub fn behind(mut self, count: usize) -> Self {
        if count > 0 {
            self.metadata.push(NodeMetadata::Behind(count));
        }
        self
    }

    pub fn draft(mut self) -> Self {
        self.metadata.push(NodeMetadata::Draft);
        self
    }
}

#[derive(Debug, Clone)]
pub enum NodeMetadata {
    Current,
    Ahead(usize),
    Behind(usize),
    Draft,
    Text(String),
}

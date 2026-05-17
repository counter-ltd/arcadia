//! Block intermediate representation for the visual Python editor.
//!
//! A [`BlockTree`] is a re-derivation of the Python source — every block carries
//! the byte range of its concrete-syntax-tree node, so the source text remains
//! the single source of truth. Anything the catalog does not model becomes a
//! [`BlockKind::Raw`] block holding verbatim source, so no input is ever lost.

/// Byte range `[start, end)` of a block's text within the source buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ByteSpan {
    pub start: usize,
    pub end: usize,
}

impl ByteSpan {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    pub fn is_empty(&self) -> bool {
        self.end <= self.start
    }
}

/// Discriminates a block. Compound kinds (those with a `:` body) own child
/// blocks; leaf kinds do not.
#[derive(Debug, Clone, PartialEq)]
pub enum BlockKind {
    /// Document root — holds top-level statements.
    Module,
    /// `def name(params):` — `header` is the full def line.
    FunctionDef { header: String },
    /// `class name(bases):` — `header` is the full class line.
    ClassDef { header: String },
    /// `if cond:` — `header` is the full if line.
    If { header: String },
    /// `elif cond:` clause attached to a preceding `if`.
    Elif { header: String },
    /// `else:` clause.
    Else,
    /// `for target in iter:` — `header` is the full for line.
    For { header: String },
    /// `while cond:` — `header` is the full while line.
    While { header: String },
    /// `with items:` — `header` is the full with line.
    With { header: String },
    /// `try:` clause.
    Try,
    /// `except ...:` clause.
    Except { header: String },
    /// `finally:` clause.
    Finally,
    /// `return value` statement.
    Return { text: String },
    /// `import ...` / `from ... import ...` statement.
    Import { text: String },
    /// Assignment statement, e.g. `x = 1`.
    Assign { text: String },
    /// Bare expression statement, e.g. a function call.
    Expr { text: String },
    /// Comment line.
    Comment { text: String },
    /// Simple keyword statement — `pass`, `break`, `continue`.
    Keyword { text: String },
    /// Unmodelled syntax — verbatim source text, preserved as-is.
    Raw { text: String },
}

impl BlockKind {
    /// True for kinds that introduce an indented body of child blocks.
    pub fn is_compound(&self) -> bool {
        matches!(
            self,
            BlockKind::Module
                | BlockKind::FunctionDef { .. }
                | BlockKind::ClassDef { .. }
                | BlockKind::If { .. }
                | BlockKind::Elif { .. }
                | BlockKind::Else
                | BlockKind::For { .. }
                | BlockKind::While { .. }
                | BlockKind::With { .. }
                | BlockKind::Try
                | BlockKind::Except { .. }
                | BlockKind::Finally
        )
    }

    /// True for clause kinds that visually attach to a preceding sibling
    /// (`elif`/`else`/`except`/`finally`).
    pub fn is_clause(&self) -> bool {
        matches!(
            self,
            BlockKind::Elif { .. }
                | BlockKind::Else
                | BlockKind::Except { .. }
                | BlockKind::Finally
        )
    }
}

/// One node in the block tree.
#[derive(Debug, Clone)]
pub struct Block {
    pub kind: BlockKind,
    pub span: ByteSpan,
    /// Body blocks for compound kinds; empty for leaves. Clause blocks
    /// (`elif`/`else`/`except`/`finally`) are appended after body blocks.
    pub children: Vec<Block>,
}

impl Block {
    pub fn leaf(kind: BlockKind, span: ByteSpan) -> Self {
        Self {
            kind,
            span,
            children: Vec::new(),
        }
    }
}

/// A parsed document — the root is always [`BlockKind::Module`].
#[derive(Debug, Clone)]
pub struct BlockTree {
    pub root: Block,
}

impl BlockTree {
    /// Total block count including the module root.
    pub fn count(&self) -> usize {
        fn walk(b: &Block) -> usize {
            1 + b.children.iter().map(walk).sum::<usize>()
        }
        walk(&self.root)
    }

    /// Resolve a child-index path (relative to the module root's children) to a
    /// block. Empty path yields the module root.
    pub fn block_at(&self, path: &[usize]) -> Option<&Block> {
        let mut node = &self.root;
        for &i in path {
            node = node.children.get(i)?;
        }
        Some(node)
    }

    /// Sibling list containing the block named by `path` (i.e. the children of
    /// its parent). `None` for an empty path (the root has no siblings).
    pub fn siblings_at(&self, path: &[usize]) -> Option<&[Block]> {
        let (_, parent_path) = path.split_last()?;
        let parent = self.block_at(parent_path)?;
        Some(&parent.children)
    }
}

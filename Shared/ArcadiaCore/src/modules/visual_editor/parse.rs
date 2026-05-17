//! Python source → [`BlockTree`] via tree-sitter.
//!
//! The parse is total: any construct the mapping does not model falls through
//! to [`BlockKind::Raw`], so arbitrary `.py` files always produce a valid tree.

use tree_sitter::{Node, Parser};

use super::blocks::{Block, BlockKind, BlockTree, ByteSpan};

/// Parse Python `source` into a block tree. Never fails — on a tree-sitter
/// error the whole document collapses to a single `Raw` block.
pub fn parse(source: &str) -> BlockTree {
    let mut parser = Parser::new();
    let lang: tree_sitter::Language = tree_sitter_python::LANGUAGE.into();
    if parser.set_language(&lang).is_err() {
        return raw_tree(source);
    }
    let Some(tree) = parser.parse(source, None) else {
        return raw_tree(source);
    };
    let children = block_body(tree.root_node(), source);
    BlockTree {
        root: Block {
            kind: BlockKind::Module,
            span: ByteSpan::new(0, source.len()),
            children,
        },
    }
}

fn raw_tree(source: &str) -> BlockTree {
    let children = if source.is_empty() {
        Vec::new()
    } else {
        vec![Block::leaf(
            BlockKind::Raw {
                text: source.to_string(),
            },
            ByteSpan::new(0, source.len()),
        )]
    };
    BlockTree {
        root: Block {
            kind: BlockKind::Module,
            span: ByteSpan::new(0, source.len()),
            children,
        },
    }
}

fn span_of(node: Node) -> ByteSpan {
    ByteSpan::new(node.start_byte(), node.end_byte())
}

fn node_text(node: Node, src: &str) -> String {
    src.get(node.start_byte()..node.end_byte())
        .unwrap_or("")
        .trim()
        .to_string()
}

/// Map every named child of a `module` or `block` node to a block.
fn block_body(node: Node, src: &str) -> Vec<Block> {
    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .map(|child| stmt_to_block(child, src))
        .collect()
}

/// First descendant child of kind `block`, preferring the `body`/`consequence`
/// field when present.
fn body_block(node: Node) -> Option<Node> {
    if let Some(b) = node.child_by_field_name("body") {
        return Some(b);
    }
    if let Some(b) = node.child_by_field_name("consequence") {
        return Some(b);
    }
    let mut cursor = node.walk();
    let found = node.children(&mut cursor).find(|c| c.kind() == "block");
    found
}

/// Header text: everything from the node start up to the body block start.
fn header_before(node: Node, body: Node, src: &str) -> String {
    let end = body.start_byte().min(node.end_byte());
    src.get(node.start_byte()..end)
        .unwrap_or("")
        .trim()
        .to_string()
}

/// Append clause blocks (`elif`/`else`/`except`/`finally`) found among `node`'s
/// children to `out`.
fn append_clauses(node: Node, src: &str, out: &mut Vec<Block>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "elif_clause" => {
                let kind = match body_block(child) {
                    Some(body) => BlockKind::Elif {
                        header: header_before(child, body, src),
                    },
                    None => BlockKind::Raw {
                        text: node_text(child, src),
                    },
                };
                out.push(compound_or_raw(child, src, kind));
            }
            "else_clause" => out.push(compound_or_raw(child, src, BlockKind::Else)),
            "except_clause" | "except_group_clause" => {
                let kind = match body_block(child) {
                    Some(body) => BlockKind::Except {
                        header: header_before(child, body, src),
                    },
                    None => BlockKind::Raw {
                        text: node_text(child, src),
                    },
                };
                out.push(compound_or_raw(child, src, kind));
            }
            "finally_clause" => out.push(compound_or_raw(child, src, BlockKind::Finally)),
            _ => {}
        }
    }
}

/// Build a compound block from `node`, or a `Raw` block if it has no body.
fn compound_or_raw(node: Node, src: &str, kind: BlockKind) -> Block {
    match body_block(node) {
        Some(body) => Block {
            kind,
            span: span_of(node),
            children: block_body(body, src),
        },
        None => Block::leaf(
            BlockKind::Raw {
                text: node_text(node, src),
            },
            span_of(node),
        ),
    }
}

fn stmt_to_block(node: Node, src: &str) -> Block {
    let span = span_of(node);
    match node.kind() {
        "function_definition" | "class_definition" | "decorated_definition" => {
            let inner_kind = inner_definition_kind(node);
            let Some(body) = body_block_for_definition(node) else {
                return Block::leaf(
                    BlockKind::Raw {
                        text: node_text(node, src),
                    },
                    span,
                );
            };
            let header = header_before(node, body, src);
            let kind = if inner_kind == "class_definition" {
                BlockKind::ClassDef { header }
            } else {
                BlockKind::FunctionDef { header }
            };
            Block {
                kind,
                span,
                children: block_body(body, src),
            }
        }
        "if_statement" => {
            let Some(consequence) = node.child_by_field_name("consequence") else {
                return raw(node, src);
            };
            let mut children = block_body(consequence, src);
            append_clauses(node, src, &mut children);
            Block {
                kind: BlockKind::If {
                    header: header_before(node, consequence, src),
                },
                span,
                children,
            }
        }
        "for_statement" => {
            let Some(body) = node.child_by_field_name("body") else {
                return raw(node, src);
            };
            let mut children = block_body(body, src);
            append_clauses(node, src, &mut children);
            Block {
                kind: BlockKind::For {
                    header: header_before(node, body, src),
                },
                span,
                children,
            }
        }
        "while_statement" => {
            let Some(body) = node.child_by_field_name("body") else {
                return raw(node, src);
            };
            let mut children = block_body(body, src);
            append_clauses(node, src, &mut children);
            Block {
                kind: BlockKind::While {
                    header: header_before(node, body, src),
                },
                span,
                children,
            }
        }
        "with_statement" => {
            let Some(body) = body_block(node) else {
                return raw(node, src);
            };
            Block {
                kind: BlockKind::With {
                    header: header_before(node, body, src),
                },
                span,
                children: block_body(body, src),
            }
        }
        "try_statement" => {
            let Some(body) = node.child_by_field_name("body") else {
                return raw(node, src);
            };
            let mut children = block_body(body, src);
            append_clauses(node, src, &mut children);
            Block {
                kind: BlockKind::Try,
                span,
                children,
            }
        }
        "return_statement" => Block::leaf(
            BlockKind::Return {
                text: node_text(node, src),
            },
            span,
        ),
        "import_statement" | "import_from_statement" | "future_import_statement" => Block::leaf(
            BlockKind::Import {
                text: node_text(node, src),
            },
            span,
        ),
        "expression_statement" => {
            let mut cursor = node.walk();
            let is_assign = node
                .named_children(&mut cursor)
                .any(|c| matches!(c.kind(), "assignment" | "augmented_assignment"));
            drop(cursor);
            let text = node_text(node, src);
            if is_assign {
                Block::leaf(BlockKind::Assign { text }, span)
            } else {
                Block::leaf(BlockKind::Expr { text }, span)
            }
        }
        "comment" => Block::leaf(
            BlockKind::Comment {
                text: node_text(node, src),
            },
            span,
        ),
        "pass_statement" | "break_statement" | "continue_statement" => Block::leaf(
            BlockKind::Keyword {
                text: node_text(node, src),
            },
            span,
        ),
        _ => raw(node, src),
    }
}

fn raw(node: Node, src: &str) -> Block {
    Block::leaf(
        BlockKind::Raw {
            text: node_text(node, src),
        },
        span_of(node),
    )
}

/// For a `decorated_definition`, the kind of the wrapped definition; otherwise
/// the node's own kind.
fn inner_definition_kind(node: Node) -> &'static str {
    if node.kind() == "decorated_definition" {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "class_definition" {
                return "class_definition";
            }
            if child.kind() == "function_definition" {
                return "function_definition";
            }
        }
    }
    if node.kind() == "class_definition" {
        "class_definition"
    } else {
        "function_definition"
    }
}

/// Body block for a definition, unwrapping `decorated_definition`.
fn body_block_for_definition(node: Node) -> Option<Node> {
    if node.kind() == "decorated_definition" {
        let mut cursor = node.walk();
        let inner = node.children(&mut cursor).find(|c| {
            matches!(c.kind(), "function_definition" | "class_definition")
        })?;
        return body_block(inner);
    }
    body_block(node)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_source_is_empty_module() {
        let tree = parse("");
        assert!(matches!(tree.root.kind, BlockKind::Module));
        assert!(tree.root.children.is_empty());
    }

    #[test]
    fn flat_statements_map_to_leaves() {
        let tree = parse("import os\nx = 1\nprint(x)\n");
        let kinds: Vec<_> = tree.root.children.iter().map(|b| &b.kind).collect();
        assert!(matches!(kinds[0], BlockKind::Import { .. }));
        assert!(matches!(kinds[1], BlockKind::Assign { .. }));
        assert!(matches!(kinds[2], BlockKind::Expr { .. }));
    }

    #[test]
    fn function_def_nests_body() {
        let tree = parse("def f(a):\n    return a\n");
        assert_eq!(tree.root.children.len(), 1);
        let def = &tree.root.children[0];
        assert!(matches!(def.kind, BlockKind::FunctionDef { .. }));
        assert_eq!(def.children.len(), 1);
        assert!(matches!(def.children[0].kind, BlockKind::Return { .. }));
    }

    #[test]
    fn if_elif_else_clauses_attach() {
        let tree = parse("if a:\n    x = 1\nelif b:\n    x = 2\nelse:\n    x = 3\n");
        let if_block = &tree.root.children[0];
        assert!(matches!(if_block.kind, BlockKind::If { .. }));
        let clause_kinds: Vec<_> = if_block
            .children
            .iter()
            .filter(|c| c.kind.is_clause())
            .map(|c| &c.kind)
            .collect();
        assert!(matches!(clause_kinds[0], BlockKind::Elif { .. }));
        assert!(matches!(clause_kinds[1], BlockKind::Else));
    }

    #[test]
    fn spans_cover_source() {
        let src = "x = 1\n";
        let tree = parse(src);
        let leaf = &tree.root.children[0];
        assert_eq!(&src[leaf.span.start..leaf.span.end], "x = 1");
    }
}

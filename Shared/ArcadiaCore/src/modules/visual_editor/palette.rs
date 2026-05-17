//! Block palette — the catalog of insertable blocks for the visual editor.
//!
//! Three sources feed the palette: built-in Python statements, built-in Arcadia
//! API calls (for authoring extensions visually), and extension-contributed
//! definitions registered through `python_registry::register_blocks`.

/// One insertable palette entry. `snippet` is Python text with 0-based relative
/// indentation; the editor re-indents every line to the insertion point.
#[derive(Clone, Debug)]
pub struct BlockDef {
    /// Unique id, e.g. `"py.if"` or `"<ext>.<block>"`.
    pub id: String,
    /// Palette display label.
    pub label: String,
    /// Grouping label, e.g. `"Python"`, `"Arcadia API"`, or an extension name.
    pub category: String,
    /// Python source inserted when the block is chosen.
    pub snippet: String,
}

impl BlockDef {
    fn new(id: &str, label: &str, category: &str, snippet: &str) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            category: category.to_string(),
            snippet: snippet.to_string(),
        }
    }
}

/// Built-in Python statement blocks.
pub fn builtin_python_blocks() -> Vec<BlockDef> {
    let c = "Python";
    vec![
        BlockDef::new("py.import", "import", c, "import module"),
        BlockDef::new("py.assign", "set", c, "name = value"),
        BlockDef::new("py.call", "call", c, "function(args)"),
        BlockDef::new("py.print", "print", c, "print(value)"),
        BlockDef::new("py.if", "if", c, "if condition:\n    pass"),
        BlockDef::new("py.for", "for", c, "for item in iterable:\n    pass"),
        BlockDef::new("py.while", "while", c, "while condition:\n    pass"),
        BlockDef::new("py.def", "def", c, "def name(params):\n    pass"),
        BlockDef::new("py.class", "class", c, "class Name:\n    pass"),
        BlockDef::new("py.return", "return", c, "return value"),
        BlockDef::new(
            "py.try",
            "try / except",
            c,
            "try:\n    pass\nexcept Exception:\n    pass",
        ),
        BlockDef::new("py.with", "with", c, "with manager as name:\n    pass"),
        BlockDef::new("py.comment", "comment", c, "# comment"),
        BlockDef::new("py.pass", "pass", c, "pass"),
    ]
}

/// Built-in Arcadia API blocks — for authoring extensions visually.
pub fn builtin_arcadia_blocks() -> Vec<BlockDef> {
    let c = "Arcadia API";
    vec![
        BlockDef::new(
            "arcadia.register_module",
            "register_module",
            c,
            "arcadia.register_module(\n    name=\"my-extension\",\n    version=\"0.1.0\",\n    description=\"What this extension does.\",\n)",
        ),
        BlockDef::new(
            "arcadia.register_command",
            "register_command",
            c,
            "arcadia.register_command(\n    \"my-extension.hello\",\n    \"Prints a greeting.\",\n    lambda args: \"hello\",\n)",
        ),
        BlockDef::new(
            "arcadia.register_blocks",
            "register_blocks",
            c,
            "arcadia.register_blocks(\"my-extension\", [\n    {\"id\": \"my-extension.block\", \"label\": \"My Block\", \"category\": \"My Extension\", \"snippet\": \"pass\"},\n])",
        ),
        BlockDef::new(
            "arcadia.execute",
            "execute",
            c,
            "result = arcadia.execute(\"shell.execute\", [\"ls\"])",
        ),
        BlockDef::new(
            "arcadia.set_timer",
            "set_timer",
            c,
            "arcadia.set_timer(1000, lambda: None)",
        ),
    ]
}

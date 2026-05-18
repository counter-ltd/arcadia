use crate::navigation;

use super::{ExecutionContext, ModuleCommand};

pub const NAME: &str = "goto";

pub fn commands() -> &'static [ModuleCommand] {
    &[ModuleCommand {
        name: "goto.page",
        description: "Resolve a navigation page by id: goto.page <page_id>",
        required_permissions: &[],
        run: run_page,
    }]
}

fn run_page(args: &[&str], _ctx: &ExecutionContext) -> String {
    let id = args.first().copied().unwrap_or("");
    match navigation::page_by_id(id) {
        Some(p) => format!("{} ({})", p.title, p.id),
        None => format!("unknown page: {id}"),
    }
}

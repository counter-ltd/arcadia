use crate::modules::{ExecutionContext, ModuleCommand};

pub const NAME: &str = "net";

fn help(_args: &[&str], context: &ExecutionContext) -> String {
    let timeout = context
        .net_timeout_ms
        .map(|ms| ms.to_string())
        .unwrap_or_else(|| "unset".to_string());
    match &context.net_as {
        Some(target) => format!(
            "Net context active: --net:as {target}, --net:timeout {timeout}"
        ),
        None => format!(
            "Net module ready. Use global flags: --net:as lan:<host/ip/alias>, --net:timeout <milliseconds> (current timeout: {timeout})"
        ),
    }
}

pub fn commands() -> &'static [ModuleCommand] {
    &[ModuleCommand {
        name: "help",
        description: "show net context and global net flag usage",
        required_permissions: &[],
        run: help,
    }]
}

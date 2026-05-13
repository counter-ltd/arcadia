use arcadia_core::modules;

pub struct CommandSpec {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub subcommands: &'static [&'static str],
}

pub const COMMAND_SPECS: &[CommandSpec] = &[
    CommandSpec {
        name: "help",
        aliases: &[],
        subcommands: &[],
    },
    CommandSpec {
        name: "ping",
        aliases: &[],
        subcommands: &[],
    },
    CommandSpec {
        name: "configuration",
        aliases: &["config", "cfg"],
        subcommands: &["show", "get", "set", "reset"],
    },
    CommandSpec {
        name: "module",
        aliases: &[],
        subcommands: &["enable", "disable"],
    },
    CommandSpec {
        name: "permit",
        aliases: &[],
        subcommands: &[],
    },
    CommandSpec {
        name: "quit",
        aliases: &[],
        subcommands: &[],
    },
];

pub fn resolve_command(command: &str) -> Option<&'static CommandSpec> {
    COMMAND_SPECS
        .iter()
        .find(|spec| spec.name == command || spec.aliases.contains(&command))
}

pub fn normalize_command(command: &str) -> String {
    resolve_command(command)
        .map(|spec| spec.name.to_string())
        .unwrap_or_else(|| command.to_string())
}

pub fn parse_execution_context(
    parts: &[String],
) -> Result<(Vec<String>, modules::ExecutionContext), String> {
    let mut cleaned = Vec::new();
    let mut net_as: Option<String> = None;
    let mut net_timeout_ms: Option<u64> = None;
    let mut i = 0;

    while i < parts.len() {
        if parts[i] == "--net:as" {
            let Some(value) = parts.get(i + 1) else {
                return Err("Usage: --net:as lan:<host/ip/alias>".to_string());
            };
            if !value.starts_with("lan:") {
                return Err(
                    "Unsupported --net:as target. Use lan:<host/ip/alias> (wan: coming later)"
                        .to_string(),
                );
            }
            net_as = Some(value.clone());
            i += 2;
            continue;
        }
        if parts[i] == "--net:timeout" {
            let Some(value) = parts.get(i + 1) else {
                return Err("Usage: --net:timeout <milliseconds>".to_string());
            };
            let parsed = value.parse::<u64>().map_err(|_| {
                "Invalid --net:timeout value. Use an integer in milliseconds".to_string()
            })?;
            net_timeout_ms = Some(parsed);
            i += 2;
            continue;
        }
        cleaned.push(parts[i].clone());
        i += 1;
    }

    Ok((
        cleaned,
        modules::ExecutionContext {
            net_as,
            net_timeout_ms,
            ..Default::default()
        },
    ))
}

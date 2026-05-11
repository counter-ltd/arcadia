mod args;
mod completion;
mod config_cmds;
mod module_cmds;

use std::cell::RefCell;
use std::io::{self, Write};
use std::sync::{OnceLock, RwLock};

use arcadia_core::config::commandline::CommandlineConfig;
use arcadia_core::config::ConfigFile;
use arcadia_core::modules;
use arcadia_core::platform;
use arcadia_core::platform::PlatformInfo;
use rustyline::history::DefaultHistory;
use rustyline::{CompletionType, Config, Editor};

use args::{normalize_command, parse_execution_context, COMMAND_SPECS};
use completion::CliHelper;
use config_cmds::handle_configuration;
use module_cmds::handle_module;

pub enum CommandResult {
    Continue,
    Quit,
}

thread_local! {
    static RESPONSE_CAPTURE: RefCell<Option<Vec<String>>> = const { RefCell::new(None) };
}

pub(super) fn settings_lock() -> &'static RwLock<CommandlineConfig> {
    static SETTINGS: OnceLock<RwLock<CommandlineConfig>> = OnceLock::new();
    SETTINGS.get_or_init(|| {
        let config = match CommandlineConfig::load_or_create() {
            Ok(config) => config,
            Err(err) => {
                eprintln!("Failed to load commandline config; using defaults: {err}");
                CommandlineConfig::default()
            }
        };
        for warning in config.color_warnings() {
            eprintln!("Warning: {warning}");
        }
        config.into()
    })
}

pub(super) fn settings() -> CommandlineConfig {
    settings_lock()
        .read()
        .map(|cfg| cfg.clone())
        .unwrap_or_else(|_| CommandlineConfig::default())
}

pub fn print_response(message: &str) {
    let captured = RESPONSE_CAPTURE.with(|capture| {
        let mut borrow = capture.borrow_mut();
        if let Some(lines) = borrow.as_mut() {
            lines.push(message.to_string());
            true
        } else {
            false
        }
    });
    if captured {
        return;
    }
    let cfg = settings();
    println!(
        "{}{}\x1b[0m {message}",
        cfg.output_ansi_code(),
        cfg.output_symbol
    );
}

pub fn print_startup(mode: &str) {
    if settings().clear_on_start {
        print!("\x1b[2J\x1b[H");
        let _ = io::stdout().flush();
    }
    println!("Arcadia base app");
    println!("Detected platform: {}", platform::current().name());
    println!("Mode: {mode}");
    println!("Status: bootstrap complete");
}

fn editor() -> Editor<CliHelper, DefaultHistory> {
    let config = Config::builder()
        .history_ignore_dups(true)
        .expect("history_ignore_dups is always configurable")
        .completion_type(CompletionType::List)
        .build();
    let mut editor =
        Editor::<CliHelper, DefaultHistory>::with_config(config).expect("editor setup failed");
    editor.set_helper(Some(CliHelper));
    editor
}

fn read_command(editor: &mut Editor<CliHelper, DefaultHistory>) -> io::Result<Option<String>> {
    let cfg = settings();
    let prompt = format!("{}{}\x1b[0m ", cfg.input_ansi_code(), cfg.input_symbol);
    match editor.readline(&prompt) {
        Ok(line) => {
            if !line.trim().is_empty() {
                let _ = editor.add_history_entry(line.as_str());
            }
            Ok(Some(line))
        }
        Err(rustyline::error::ReadlineError::Interrupted) => Ok(Some(String::new())),
        Err(rustyline::error::ReadlineError::Eof) => Ok(None),
        Err(err) => Err(io::Error::other(err)),
    }
}

pub fn start_loop(quit: impl FnOnce() + Copy) {
    let mut editor = editor();
    loop {
        match read_command(&mut editor) {
            Ok(None) => break,
            Ok(Some(line)) => {
                if let CommandResult::Quit = handle(&line) {
                    quit();
                    break;
                }
            }
            Err(err) => {
                eprintln!("CLI input error: {err}");
                break;
            }
        }
    }
}

pub fn handle(input: &str) -> CommandResult {
    handle_with(input, print_response)
}

pub fn handle_internal(input: &str) -> String {
    RESPONSE_CAPTURE.with(|capture| {
        *capture.borrow_mut() = Some(Vec::new());
    });
    let _ = handle(input);
    RESPONSE_CAPTURE.with(|capture| capture.borrow_mut().take().unwrap_or_default().join("\n"))
}

fn handle_permit(parts: &[String]) -> Result<String, String> {
    use arcadia_core::modules::permissions;

    fn parse_bool(s: &str) -> Result<bool, String> {
        match s.to_ascii_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => Ok(true),
            "false" | "0" | "no" | "off" => Ok(false),
            _ => Err(format!("Expected true or false, got: {s}")),
        }
    }

    match parts.len() {
        2 => {
            let perm = parts[0].as_str();
            let b = parse_bool(&parts[1])?;
            permissions::apply_global_set(perm, b)
        }
        3 => {
            let subj = permissions::normalize_cli_subject(&parts[0])?;
            let perm = parts[1].as_str();
            let b = parse_bool(&parts[2])?;
            permissions::apply_grant_set(&subj, perm, b)
        }
        _ => Err(
            "Usage: permit <permission-id> <true|false>\n       permit <subject> <permission-id> <true|false>\nSubjects: module:<name>, python:<id>, or a bare registry module / extension id."
                .to_string(),
        ),
    }
}

fn handle_with(input: &str, mut respond: impl FnMut(&str)) -> CommandResult {
    let trimmed = input.trim();
    let mut parts = trimmed
        .split_whitespace()
        .map(str::to_string)
        .collect::<Vec<_>>();

    let (parsed_parts, exec_ctx) = match parse_execution_context(&parts) {
        Ok(value) => value,
        Err(err) => {
            respond(&err);
            return CommandResult::Continue;
        }
    };
    parts = parsed_parts;

    if let Some(first) = parts.first_mut() {
        *first = normalize_command(first);
    }

    if !parts.is_empty() && parts[0] == "permit" {
        match handle_permit(&parts[1..]) {
            Ok(msg) => respond(&msg),
            Err(err) => respond(&err),
        }
        return CommandResult::Continue;
    }

    if !parts.is_empty() && parts[0] == "configuration" {
        let part_refs = parts.iter().map(String::as_str).collect::<Vec<_>>();
        handle_configuration(&part_refs);
        return CommandResult::Continue;
    }
    if !parts.is_empty() && parts[0] == "module" {
        let part_refs = parts.iter().map(String::as_str).collect::<Vec<_>>();
        handle_module(&part_refs);
        return CommandResult::Continue;
    }

    if let Some(first) = parts.first().map(String::as_str) {
        if first.contains('.') {
            let args = parts.iter().skip(1).map(String::as_str).collect::<Vec<_>>();
            match modules::execute_command(first, &args, &exec_ctx) {
                Ok(Some(message)) => {
                    respond(&message);
                    return CommandResult::Continue;
                }
                Ok(None) => {}
                Err(err) => {
                    respond(&err);
                    return CommandResult::Continue;
                }
            }
        }
    }
    if parts.len() >= 2 {
        let composed = format!("{}.{}", parts[0], parts[1]);
        let args = parts.iter().skip(2).map(String::as_str).collect::<Vec<_>>();
        match modules::execute_command(&composed, &args, &exec_ctx) {
            Ok(Some(message)) => {
                respond(&message);
                return CommandResult::Continue;
            }
            Ok(None) => {}
            Err(err) => {
                respond(&err);
                return CommandResult::Continue;
            }
        }
    }

    match parts.first().map(String::as_str).unwrap_or("") {
        "help" => {
            for line in help_lines() {
                respond(&line);
            }
            CommandResult::Continue
        }
        "ping" => {
            respond("pong");
            CommandResult::Continue
        }
        "quit" => CommandResult::Quit,
        "" => CommandResult::Continue,
        _ => {
            respond(&format!("Unknown command: {trimmed}"));
            CommandResult::Continue
        }
    }
}

fn help_lines() -> Vec<String> {
    let mut lines = vec!["Available commands:".to_string()];
    for spec in COMMAND_SPECS {
        match spec.name {
            "help" => lines.push("- help: show this help message".to_string()),
            "ping" => lines.push("- ping: respond with pong".to_string()),
            "quit" => lines.push("- quit: exit Arcadia".to_string()),
            "configuration" => {
                lines
                    .push("- configuration <name>: open config file in default editor".to_string());
                lines.push(
                    "- configuration [show|get|set|reset] ...: manage commandline config"
                        .to_string(),
                );
                if !spec.aliases.is_empty() {
                    let aliases = spec.aliases.join(" -> ");
                    lines.push(format!("- aliases: {aliases} -> configuration"));
                }
            }
            "module" => {
                lines.push("- module <name> enable|disable: toggle a module".to_string());
                lines.push(
                    "- module <name> enable -requirements: enable module and required dependencies"
                        .to_string(),
                );
            }
            "permit" => {
                lines.push(
                    "- permit <permission-id> <true|false>: set a global permission flag".to_string(),
                );
                lines.push(
                    "- permit <subject> <permission-id> <true|false>: set per-subject grant (module:<name>, python:<id>, or bare id)"
                        .to_string(),
                );
            }
            _ => {}
        }
    }
    let module_command_lines = modules::enabled_command_help_lines();
    if !module_command_lines.is_empty() {
        lines.push("- enabled module commands:".to_string());
        lines.extend(module_command_lines);
    }
    lines.push(
        "- global flags: --net:as lan:<host/ip/alias> | --net:timeout <milliseconds>".to_string(),
    );
    lines
}

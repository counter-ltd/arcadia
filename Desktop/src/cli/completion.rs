use std::borrow::Cow::{self, Borrowed, Owned};

use arcadia_core::modules;
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};

use arcadia_core::config::permissions::PERMISSION_REGISTRY;

use super::args::{normalize_command, resolve_command, COMMAND_SPECS};
use super::config_cmds::{modules_keys, provider_names, scoped_key_candidates};

#[derive(Default)]
pub struct CliHelper;

impl Helper for CliHelper {}
impl Validator for CliHelper {}

impl Hinter for CliHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Option<Self::Hint> {
        if pos != line.len() {
            return None;
        }
        let (start, suggestions) = completion_candidates(line, pos);
        let typed = &line[start..pos];
        if typed.trim().is_empty() {
            return None;
        }
        suggestions
            .iter()
            .find(|c| c.starts_with(typed) && c.as_str() != typed)
            .map(|c| c[typed.len()..].to_string())
    }
}

impl Completer for CliHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Self::Candidate>), ReadlineError> {
        let (start, suggestions) = completion_candidates(line, pos);
        let prefix = line[start..pos].to_ascii_lowercase();
        let pairs = suggestions
            .into_iter()
            .filter(|c| c.starts_with(&prefix))
            .map(|c| Pair {
                display: c.to_string(),
                replacement: c.to_string(),
            })
            .collect::<Vec<_>>();
        Ok((start, pairs))
    }
}

impl Highlighter for CliHelper {
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned(format!("\x1b[90m{hint}\x1b[0m"))
    }

    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        Borrowed(line)
    }
}

pub fn root_command_tokens() -> Vec<String> {
    let mut tokens = COMMAND_SPECS
        .iter()
        .flat_map(|spec| std::iter::once(spec.name).chain(spec.aliases.iter().copied()))
        .map(str::to_string)
        .collect::<Vec<_>>();
    tokens.extend(modules::enabled_module_names());
    tokens.extend(modules::enabled_command_tokens());
    tokens
}

pub fn completion_candidates(line: &str, pos: usize) -> (usize, Vec<String>) {
    let head = &line[..pos];
    let ends_with_space = head.chars().last().is_some_and(char::is_whitespace);
    let tokens = head.split_whitespace().collect::<Vec<_>>();

    if tokens.is_empty() {
        return (0, root_command_tokens());
    }

    if tokens.len() == 1 && !ends_with_space {
        let start = head.rfind(char::is_whitespace).map_or(0, |idx| idx + 1);
        return (start, root_command_tokens());
    }

    let command = normalize_command(tokens[0]);
    let active_index = if ends_with_space {
        tokens.len()
    } else {
        tokens.len().saturating_sub(1)
    };
    let start = head.rfind(char::is_whitespace).map_or(0, |idx| idx + 1);

    let suggestions = match command.as_str() {
        "configuration" => match active_index {
            1 => resolve_command("configuration")
                .map(|spec| spec.subcommands.iter().map(|v| (*v).to_string()).collect())
                .unwrap_or_default(),
            2 => match tokens.get(1).copied() {
                Some("show") => provider_names(),
                Some("get") | Some("set") | Some("reset") => provider_names()
                    .into_iter()
                    .chain(scoped_key_candidates())
                    .collect(),
                _ => Vec::new(),
            },
            _ => Vec::new(),
        },
        "module" => match active_index {
            1 => modules_keys().unwrap_or_default(),
            2 => vec!["enable".to_string(), "disable".to_string()],
            3 if tokens.get(2).copied() == Some("enable") => vec!["-requirements".to_string()],
            _ => Vec::new(),
        },
        "permit" => match active_index {
            1 => {
                let mut v: Vec<String> =
                    PERMISSION_REGISTRY.iter().map(|p| p.id.to_string()).collect();
                if let Ok(mk) = modules_keys() {
                    v.extend(mk.into_iter().map(|k| format!("module:{k}")));
                }
                v
            }
            2 => {
                if let Some(first) = tokens.get(1).copied() {
                    if PERMISSION_REGISTRY.iter().any(|p| p.id == first) {
                        return (start, vec!["true".to_string(), "false".to_string()]);
                    }
                }
                PERMISSION_REGISTRY.iter().map(|p| p.id.to_string()).collect()
            }
            3 => vec!["true".to_string(), "false".to_string()],
            _ => Vec::new(),
        },
        other => match active_index {
            1 => modules::enabled_module_command_names(other),
            _ => Vec::new(),
        },
    };

    (start, suggestions)
}

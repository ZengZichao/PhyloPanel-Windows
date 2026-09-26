use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

const TYPE_WORDS: &[(&str, FlagKind)] = &[
    ("int", FlagKind::Int),
    ("int8", FlagKind::Int),
    ("int16", FlagKind::Int),
    ("int32", FlagKind::Int),
    ("int64", FlagKind::Int),
    ("uint", FlagKind::Int),
    ("uint8", FlagKind::Int),
    ("uint16", FlagKind::Int),
    ("uint32", FlagKind::Int),
    ("uint64", FlagKind::Int),
    ("count", FlagKind::Int),
    ("float32", FlagKind::Float),
    ("float64", FlagKind::Float),
    ("float", FlagKind::Float),
    ("string", FlagKind::String),
    ("stringSlice", FlagKind::StringSlice),
    ("stringArray", FlagKind::StringSlice),
    ("intSlice", FlagKind::StringSlice),
    ("duration", FlagKind::Duration),
    ("bool", FlagKind::Bool),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FlagKind {
    Bool,
    Int,
    Float,
    String,
    StringSlice,
    Duration,
}

/// Semantic role. GoTree itself is inconsistent about whether `--input` is a local or a
/// persistent flag, so the UI keys off this rather than off which cobra block it came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FlagRole {
    Input,
    Output,
    Format,
    File,
    Other,
}

fn role_of(name: &str) -> FlagRole {
    match name {
        "input" => FlagRole::Input,
        "output" => FlagRole::Output,
        "format" => FlagRole::Format,
        _ if name.contains("file") => FlagRole::File,
        _ => FlagRole::Other,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlagSpec {
    pub name: String,
    pub short: Option<String>,
    pub kind: FlagKind,
    pub role: FlagRole,
    pub default: Option<String>,
    pub description: String,
    pub choices: Option<Vec<String>>,
    pub inherited: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandSpec {
    /// Space separated invocation path, e.g. "draw svg".
    pub id: String,
    pub path: Vec<String>,
    /// First path element, used as the coarse filter in the UI.
    pub group: String,
    pub description: String,
    pub flags: Vec<FlagSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolSchema {
    pub binary: String,
    pub version: String,
    pub commands: Vec<CommandSpec>,
}

struct HelpInfo {
    own: Vec<FlagSpec>,
    inherited: Vec<FlagSpec>,
    runnable: bool,
    is_group: bool,
    description: String,
}

pub struct Reflector {
    exe: PathBuf,
}

impl Reflector {
    pub fn new(exe: impl AsRef<Path>) -> Self {
        Self { exe: exe.as_ref().to_path_buf() }
    }

    fn run(&self, args: &[String]) -> Result<String, String> {
        let mut cmd = Command::new(&self.exe);
        cmd.args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);

        let out = cmd
            .output()
            .map_err(|e| format!("Cannot launch {}: {}", self.exe.display(), e))?;
        if out.stdout.is_empty() && !out.status.success() {
            return Err(format!(
                "{} exited with {}: {}",
                self.exe.display(),
                out.status,
                String::from_utf8_lossy(&out.stderr).trim()
            ));
        }
        Ok(strip_ansi(&String::from_utf8_lossy(&out.stdout)))
    }

    fn complete(&self, path: &[String]) -> Result<Vec<String>, String> {
        let mut args = vec!["__complete".to_string()];
        args.extend(path.iter().cloned());
        args.push(String::new());
        let out = self.run(&args)?;
        Ok(out
            .lines()
            .filter_map(|l| {
                let l = l.trim_end();
                if l.is_empty() || l.starts_with(':') {
                    return None;
                }
                let name = match l.split_once('\t') {
                    Some((n, _)) => n.trim(),
                    None => l.trim(),
                };
                if name.is_empty() || name.contains(' ') || hidden_command(name) {
                    return None;
                }
                Some(name.to_string())
            })
            .collect())
    }

    pub fn reflect(&self) -> Result<ToolSchema, String> {
        let mut commands = Vec::new();
        for name in self.complete(&[])? {
            self.walk(&mut vec![name], &mut commands)?;
        }
        commands.sort_by(|a, b| a.id.cmp(&b.id));

        Ok(ToolSchema {
            binary: self.exe.display().to_string(),
            version: self.version(),
            commands,
        })
    }

    fn version(&self) -> String {
        let mut cmd = Command::new(&self.exe);
        cmd.arg("version")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);
        // gotree prints its version to stderr, so neither stream can be trusted alone.
        cmd.output()
            .ok()
            .and_then(|o| {
                let streams = [
                    String::from_utf8_lossy(&o.stdout).to_string(),
                    String::from_utf8_lossy(&o.stderr).to_string(),
                ];
                streams
                    .iter()
                    .flat_map(|s| s.lines())
                    .map(|l| l.trim())
                    .find(|l| {
                        !l.is_empty()
                            && !l.starts_with("unknown")
                            && !l.starts_with("Error")
                            && !l.starts_with("Completion")
                    })
                    .map(|l| l.to_string())
            })
            .unwrap_or_default()
    }

    fn walk(&self, path: &mut Vec<String>, out: &mut Vec<CommandSpec>) -> Result<(), String> {
        let mut help_args = path.clone();
        help_args.push("--help".to_string());
        let info = parse_help(&self.run(&help_args)?);
        let children = if info.is_group { self.complete(path)? } else { Vec::new() };

        // A cobra command can be runnable *and* own subcommands, e.g. `gotree cut`.
        if info.runnable {
            let mut flags = info.own;
            for f in info.inherited {
                if !flags.iter().any(|e| e.name == f.name) {
                    flags.push(f);
                }
            }
            out.push(CommandSpec {
                id: path.join(" "),
                path: path.clone(),
                group: path[0].clone(),
                description: info.description,
                flags,
            });
        }

        for name in children {
            path.push(name);
            self.walk(path, out)?;
            path.pop();
        }
        Ok(())
    }
}

fn hidden_command(name: &str) -> bool {
    matches!(name, "help" | "completion" | "shell") || name.starts_with("__")
}

fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\u{1b}' {
            out.push(c);
            continue;
        }
        if chars.peek() == Some(&'[') {
            chars.next();
            for c in chars.by_ref() {
                if c.is_ascii_alphabetic() {
                    break;
                }
            }
        }
    }
    out
}

fn parse_help(help: &str) -> HelpInfo {
    let mut sections: Vec<(String, Vec<String>)> = Vec::new();
    for line in help.lines() {
        let trimmed = line.trim();
        let is_header = !line.starts_with(char::is_whitespace)
            && trimmed.len() > 1
            && trimmed.ends_with(':');
        if is_header {
            sections.push((trimmed.trim_end_matches(':').to_string(), Vec::new()));
        } else {
            match sections.last_mut() {
                Some((_, lines)) => lines.push(line.to_string()),
                None => sections.push((String::new(), vec![line.to_string()])),
            }
        }
    }

    let take = |needle: &str| -> Vec<String> {
        sections
            .iter()
            .find(|(n, _)| n == needle)
            .map(|(_, l)| l.clone())
            .unwrap_or_default()
    };

    let usage = take("Usage");
    HelpInfo {
        own: parse_flag_lines(&take("Flags"), false),
        inherited: parse_flag_lines(&take("Global Flags"), true),
        runnable: usage
            .iter()
            .any(|l| l.contains("[flags]") || l.contains("[required flags]")),
        is_group: usage.iter().any(|l| l.contains("[command]")),
        description: sections
            .iter()
            .find(|(n, _)| n.is_empty())
            .map(|(_, l)| {
                l.iter()
                    .map(|s| s.trim_end())
                    .filter(|s| !s.trim().is_empty())
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .unwrap_or_default(),
    }
}

fn parse_flag_lines(lines: &[String], inherited: bool) -> Vec<FlagSpec> {
    let mut flags: Vec<FlagSpec> = Vec::new();
    for raw in lines {
        if raw.trim().is_empty() {
            continue;
        }
        let t = raw.trim_start();
        let short_form = t
            .get(0..2)
            .map(|p| p.starts_with('-') && p[1..].starts_with(char::is_alphanumeric))
            .unwrap_or(false);
        let is_entry = t.starts_with("--") || (short_form && t.contains(", --"));

        if !is_entry {
            if let Some(last) = flags.last_mut() {
                last.description.push('\n');
                last.description.push_str(t);
            }
            continue;
        }

        let (short, after_short) = match t.find(", --") {
            Some(pos) => (Some(t[1..pos].to_string()), &t[pos + 2..]),
            None => (None, t),
        };
        let body = match after_short.strip_prefix("--") {
            Some(b) => b,
            None => continue,
        };
        let (name, rest) = match body.split_once(char::is_whitespace) {
            Some((n, r)) => (n.to_string(), r),
            None => (body.to_string(), ""),
        };
        if name == "help" {
            continue;
        }

        let rest = rest.trim_start();
        let first_word = rest.split_whitespace().next().unwrap_or("");
        let (kind, mut description) = match TYPE_WORDS.iter().find(|(w, _)| *w == first_word) {
            Some((word, kind)) => (*kind, rest[word.len()..].trim_start().to_string()),
            None => (FlagKind::Bool, rest.to_string()),
        };

        let default = take_default(&mut description, kind);
        let choices = take_choices(&mut description);

        flags.push(FlagSpec {
            name: name.clone(),
            short,
            kind,
            role: role_of(&name),
            default,
            description: description.trim_end().to_string(),
            choices,
            inherited,
        });
    }
    flags
}

/// Pulls pflag's trailing `(default ...)`. Bool flags are skipped so that author-written
/// prose like "(default : normal)" stays out of the structured field.
fn take_default(description: &mut String, kind: FlagKind) -> Option<String> {
    if kind == FlagKind::Bool || !description.ends_with(')') {
        return None;
    }
    let open = description.rfind('(')?;
    let inner = description[open + 1..description.len() - 1].to_string();
    let value = match inner.strip_prefix("default ") {
        Some(v) => v.trim().to_string(),
        None => return None,
    };
    if value.is_empty() || value.starts_with(':') {
        return None;
    }
    *description = description[..open].trim_end().to_string();
    Some(value.trim_matches('"').to_string())
}

/// Recovers enums cobra only exposes as prose, e.g.
/// `Input tree format (newick, nexus, phyloxml, or nextstrain)`.
fn take_choices(description: &mut String) -> Option<Vec<String>> {
    if !description.ends_with(')') {
        return None;
    }
    let open = description.rfind('(')?;
    let inner = description[open + 1..description.len() - 1].to_string();
    if !inner.contains(',') {
        return None;
    }
    let choices: Vec<String> = inner
        .split(',')
        .map(|c| {
            c.trim()
                .trim_start_matches("or ")
                .trim_start_matches("and ")
                .trim()
                .to_string()
        })
        .filter(|c| !c.is_empty() && c.len() < 40 && !c.chars().any(|ch| ch.is_uppercase()))
        .collect();
    if choices.len() < 2 {
        return None;
    }
    *description = description[..open].trim_end().to_string();
    Some(choices)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SVG_HELP: &str = "Draw trees in svg files.\n\nUsage:\n  gotree draw svg [flags]\n\nFlags:\n  -c, --circular     Circular/Polar layout (default : normal)\n  -H, --height int   Height of svg image in pixels (default 200)\n  -h, --help         help for svg\n  -w, --width int    Width of svg image in pixels (default 200)\n\nGlobal Flags:\n      --format string   Input tree format (newick, nexus, phyloxml, or nextstrain) (default \"newick\")\n  -i, --input string    Input tree (default \"stdin\")\n";

    #[test]
    fn parses_typed_flag_block() {
        let info = parse_help(SVG_HELP);
        assert!(info.runnable);
        assert!(!info.is_group);
        assert_eq!(info.own.len(), 3);

        let circular = info.own.iter().find(|f| f.name == "circular").unwrap();
        assert_eq!(circular.kind, FlagKind::Bool);
        assert_eq!(circular.short.as_deref(), Some("c"));
        assert!(circular.default.is_none(), "prose default must not leak");

        let height = info.own.iter().find(|f| f.name == "height").unwrap();
        assert_eq!(height.kind, FlagKind::Int);
        assert_eq!(height.default.as_deref(), Some("200"));

        let format = info.inherited.iter().find(|f| f.name == "format").unwrap();
        assert_eq!(format.role, FlagRole::Format);
        assert_eq!(
            format.choices.as_deref().unwrap(),
            ["newick", "nexus", "phyloxml", "nextstrain"]
        );
    }

    #[test]
    fn distinguishes_group_and_dual_command() {
        assert!(parse_help("Usage:\n  gotree draw [command]\n").is_group);
        let dual = parse_help("Usage:\n  gotree cut [flags]\n  gotree cut [command]\n");
        assert!(dual.runnable && dual.is_group);
    }
}

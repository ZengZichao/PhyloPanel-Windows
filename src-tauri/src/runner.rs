use base64::Engine;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Child, ChildStdout, Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use crate::error;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Step {
    /// Command path as typed on the CLI, e.g. "draw svg".
    pub command: String,
    #[serde(default)]
    pub flags: BTreeMap<String, serde_json::Value>,
    /// Bare arguments. Cobra does not advertise which commands read them, so the tool
    /// pack names them and the UI leaves this as a free-text escape hatch.
    #[serde(default)]
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StepResult {
    pub command: String,
    pub argv: Vec<String>,
    pub code: Option<i32>,
    pub ok: bool,
    pub stderr: String,
    /// Clean message pulled out of the tool's `[Error] in cmd/... , message: ...` line.
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunResult {
    pub command_line: String,
    pub stdout: String,
    pub stdout_base64: Option<String>,
    pub is_binary: bool,
    pub elapsed_ms: u128,
    pub steps: Vec<StepResult>,
}

/// Builds the argv for one step. Flags that only restate the CLI default are left out:
/// booleans appear solely when true, and empty strings are dropped.
pub fn build_argv(step: &Step) -> Vec<String> {
    let mut argv: Vec<String> = step.command.split_whitespace().map(|s| s.to_string()).collect();
    for (name, value) in &step.flags {
        match value {
            serde_json::Value::Bool(true) => argv.push(format!("--{}", name)),
            serde_json::Value::String(s) if !s.is_empty() => {
                argv.push(format!("--{}", name));
                argv.push(s.clone());
            }
            serde_json::Value::Number(n) => {
                argv.push(format!("--{}", name));
                argv.push(n.to_string());
            }
            serde_json::Value::Array(items) => {
                let joined = items
                    .iter()
                    .filter_map(|i| i.as_str().map(|s| s.to_string()))
                    .collect::<Vec<_>>()
                    .join(",");
                if !joined.is_empty() {
                    argv.push(format!("--{}", name));
                    argv.push(joined);
                }
            }
            _ => {}
        }
    }
    argv.extend(step.args.iter().filter(|a| !a.is_empty()).cloned());
    argv
}

pub fn shell_quote(arg: &str) -> String {
    if arg.is_empty() {
        return "''".to_string();
    }
    if arg
        .chars()
        .all(|c| c.is_alphanumeric() || matches!(c, '.' | '_' | '-' | '/' | ':' | '=' | ',' | '@'))
    {
        return arg.to_string();
    }
    format!("'{}'", arg.replace('\'', "'\\''"))
}

/// The command line a user can paste into a terminal. Nothing the panel can do should be
/// hidden from the CLI, so this is rendered in the UI next to every result.
pub fn command_line(steps: &[Step]) -> String {
    steps
        .iter()
        .map(|s| {
            build_argv(s)
                .iter()
                .map(|a| shell_quote(a))
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect::<Vec<_>>()
        .join(" | ")
}

fn tidy_error(stderr: &str) -> Option<String> {
    for line in stderr.lines() {
        let l = line.trim();
        if let Some(pos) = l.find("message: ") {
            let rest = l[pos + "message: ".len()..].trim();
            if !rest.is_empty() {
                return Some(rest.to_string());
            }
        }
        if l.starts_with("[Error]") || l.starts_with("Error:") {
            let rest = l
                .trim_start_matches("[Error]")
                .trim_start_matches("Error:")
                .trim();
            if !rest.is_empty() {
                return Some(rest.to_string());
            }
        }
    }
    None
}

fn read_to_end(mut pipe: ChildStdout, tx: Sender<Vec<u8>>) {
    let mut buf = Vec::new();
    let _ = pipe.read_to_end(&mut buf);
    let _ = tx.send(buf);
}

/// Runs `steps` as an OS-level pipeline: step N's stdout is wired straight into step N+1's
/// stdin, so no intermediate result ever lands in our process or on disk.
pub fn run_pipeline(
    binary: impl AsRef<Path>,
    steps: &[Step],
    input_text: Option<&str>,
) -> Result<RunResult, String> {
    if steps.is_empty() {
        return Err(error("noSteps", ""));
    }
    let started = std::time::Instant::now();
    let binary = binary.as_ref();

    let mut children: Vec<Child> = Vec::with_capacity(steps.len());
    let mut stderr_rx: Vec<Receiver<Vec<u8>>> = Vec::with_capacity(steps.len());
    let mut stdout_rx: Option<Receiver<Vec<u8>>> = None;

    for (index, step) in steps.iter().enumerate() {
        let argv = build_argv(step);
        let mut cmd = Command::new(binary);
        cmd.args(&argv).stderr(Stdio::piped()).stdout(Stdio::piped());
        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);

        if index == 0 {
            match input_text {
                Some(_) => cmd.stdin(Stdio::piped()),
                None => cmd.stdin(Stdio::null()),
            };
        } else {
            let upstream = children[index - 1]
                .stdout
                .take()
                .ok_or_else(|| error("noUpstream", ""))?;
            cmd.stdin(Stdio::from(upstream));
        }

        let mut child = cmd
            .spawn()
            .map_err(|e| error("spawn", format_args!("{}: {}", binary.display(), e)))?;

        let (err_tx, err_rx) = mpsc::channel();
        stderr_rx.push(err_rx);
        if let Some(pipe) = child.stderr.take() {
            thread::spawn(move || {
                let mut buf = Vec::new();
                let mut pipe = pipe;
                let _ = pipe.read_to_end(&mut buf);
                let _ = err_tx.send(buf);
            });
        }

        if index == 0 {
            if let (Some(text), Some(mut sink)) = (input_text.map(str::to_string), child.stdin.take()) {
                thread::spawn(move || {
                    let _ = sink.write_all(text.as_bytes());
                    let _ = sink.flush();
                });
            }
        }
        if index + 1 == steps.len() {
            if let Some(pipe) = child.stdout.take() {
                let (tx, rx) = mpsc::channel();
                thread::spawn(move || read_to_end(pipe, tx));
                stdout_rx = Some(rx);
            }
        }
        children.push(child);
    }

    let statuses: Vec<std::process::ExitStatus> = children
        .iter_mut()
        .map(|c| c.wait())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| error("wait", e))?;

    let raw = stdout_rx.map(|rx| rx.recv().unwrap_or_default()).unwrap_or_default();

    let mut step_results = Vec::with_capacity(steps.len());
    for (index, status) in statuses.into_iter().enumerate() {
        let stderr = String::from_utf8_lossy(
            &stderr_rx[index].recv().unwrap_or_default(),
        )
        .to_string();
        step_results.push(StepResult {
            command: steps[index].command.clone(),
            argv: build_argv(&steps[index]),
            ok: status.success(),
            code: status.code(),
            message: if status.success() { None } else { tidy_error(&stderr) },
            stderr,
        });
    }

    let (stdout, stdout_base64, is_binary) = match String::from_utf8(raw.clone()) {
        Ok(text) => (text, None, false),
        Err(_) => (
            String::new(),
            Some(base64::engine::general_purpose::STANDARD.encode(&raw)),
            true,
        ),
    };

    Ok(RunResult {
        command_line: command_line(steps),
        stdout,
        stdout_base64,
        is_binary,
        elapsed_ms: started.elapsed().as_millis(),
        steps: step_results,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flags(pairs: &[(&str, serde_json::Value)]) -> BTreeMap<String, serde_json::Value> {
        pairs.iter().map(|(k, v)| (k.to_string(), v.clone())).collect()
    }

    #[test]
    fn only_material_flags_reach_argv() {
        let argv = build_argv(&Step {
            command: "draw svg".into(),
            flags: flags(&[
                ("circular", serde_json::json!(true)),
                ("radial", serde_json::json!(false)),
                ("width", serde_json::json!(800)),
                ("output", serde_json::json!("")),
                ("format", serde_json::json!("nexus")),
            ]),
            args: vec![],
        });
        assert_eq!(
            argv,
            ["draw", "svg", "--circular", "--format", "nexus", "--width", "800"]
        );
    }

    #[test]
    fn quotes_arguments_that_need_it() {
        assert_eq!(shell_quote("tree.tre"), "tree.tre");
        assert_eq!(shell_quote("a b"), "'a b'");
        assert_eq!(
            command_line(&[step_args("stats", &[("input", serde_json::json!("my tree.tre"))], &[])]),
            "stats --input 'my tree.tre'"
        );
    }

    #[test]
    fn extracts_the_real_message_from_gotree_noise() {
        let stderr = "[Error] in cmd/reroot.go (line 42), message: cannot find outgroup: Node \"xyz\" is not in the tree\n\nUsage:\n  gotree reroot outgroup [flags]\n";
        assert_eq!(
            tidy_error(stderr).as_deref(),
            Some("cannot find outgroup: Node \"xyz\" is not in the tree")
        );
    }

    fn step(command: &str, pairs: &[(&str, serde_json::Value)]) -> Step {
        step_args(command, pairs, &[])
    }

    fn step_args(
        command: &str,
        pairs: &[(&str, serde_json::Value)],
        args: &[&str],
    ) -> Step {
        Step {
            command: command.into(),
            flags: flags(pairs),
            args: args.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Set GOTREE_BIN to exercise the real pipeline; skipped otherwise so the suite stays
    /// runnable without a checkout of gotree.
    fn gotree_binary() -> Option<std::path::PathBuf> {
        std::env::var("GOTREE_BIN").ok().map(std::path::PathBuf::from).filter(|p| p.exists())
    }

    #[test]
    fn chains_generate_into_draw() {
        let Some(exe) = gotree_binary() else { return };
        let result = run_pipeline(
            &exe,
            &[
                step("generate yuletree", &[("nbtips", serde_json::json!(5)), ("seed", serde_json::json!(1))]),
                step("draw svg", &[("width", serde_json::json!(400)), ("height", serde_json::json!(400))]),
            ],
            None,
        )
        .expect("pipeline");
        assert!(result.steps.iter().all(|s| s.ok), "{:?}", result.steps);
        assert!(result.stdout.contains("<svg"), "not svg: {}", &result.stdout[..80.min(result.stdout.len())]);
    }

    #[test]
    fn feeds_input_text_into_a_multi_step_chain() {
        let Some(exe) = gotree_binary() else { return };
        let newick = "(1:0.1,(2:0.2,(3:0.3,4:0.4,5:0.5,6:0.6)poly:0.7)int:0.8)root;\n";
        let result = run_pipeline(
            &exe,
            &[
                step("reroot midpoint", &[]),
                step("draw text", &[("width", serde_json::json!(40))]),
            ],
            Some(newick),
        )
        .expect("pipeline");
        assert!(result.steps.iter().all(|s| s.ok), "{:?}", result.steps);
        assert!(result.stdout.contains("6"), "{}", result.stdout);
        assert!(result.stdout.lines().count() > 3, "{}", result.stdout);
        assert_eq!(result.command_line, "reroot midpoint | draw text --width 40");

        // `stats` consumes the tree instead of passing it through, so it ends a chain.
        let stats = run_pipeline(
            &exe,
            &[step("reroot midpoint", &[]), step("stats", &[])],
            Some(newick),
        )
        .expect("pipeline");
        assert!(stats.stdout.contains("tips"), "{}", stats.stdout);
    }

    #[test]
    fn positional_arguments_reach_the_steps_that_need_them() {
        let Some(exe) = gotree_binary() else { return };
        let ok = run_pipeline(
            &exe,
            &[step_args(
                "reroot outgroup",
                &[("remove-outgroup", serde_json::json!(true))],
                &["3", "4", "5", "6"],
            )],
            Some("(1,(2,(3,4,5,6)poly)int)root;\n"),
        )
        .expect("pipeline");
        assert!(ok.steps[0].ok, "stderr: {}", ok.steps[0].stderr);
        assert!(ok.stdout.contains('1'), "{}", ok.stdout);

        let bad = run_pipeline(
            &exe,
            &[step("reroot outgroup", &[]), step("stats", &[])],
            Some("(1,(2,3)int)root;\n"),
        )
        .expect("pipeline");
        assert!(!bad.steps[0].ok, "missing tips must fail");
        assert_eq!(bad.steps[0].message.as_deref(), Some("Not group given"));
    }
}

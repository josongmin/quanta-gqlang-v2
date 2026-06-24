#![cfg_attr(
    test,
    allow(
        clippy::expect_used,
        clippy::unwrap_used,
        clippy::indexing_slicing,
        clippy::panic,
        reason = "unit-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
    )
)]

//! QGLang CLI front door (v3-ONLY).
//!
//! HARD CUTOVER: the CLI now drives EXACTLY the one shared v3 compiler pipeline
//! (`quanta-gqlang-v2-pipeline`) over the one shared v3 surface parser/formatter
//! (`quanta-gqlang-v2-surface`). The legacy v2 `syntax -> sema -> ir` direct
//! verbs (`parse`, `sema`, `explain`, `doctor`, `headless`, `zero-query`) are
//! gone; `query` and `fmt` are now the v3 path and `v3-query` / `v3-fmt` are
//! retained as explicit aliases for the same behavior.
//!
//! - `query` / `v3-query`: compile the input through
//!   `pipeline::compile_program_structural` (single item) or
//!   `pipeline::compile_document_structural` over `surface::parse_document`
//!   (multi-item, `;`-separated) and render the per-family plan + cost summary.
//! - `fmt` / `v3-fmt`: parse through `surface::parse_program` /
//!   `surface::parse_document` and render canonical surface via
//!   `surface::format_item` / `surface::format_document`. Idempotent.

use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::process;

use quanta_gqlang_v2_cli::commands;

const USAGE_V1: &str = concat!(
    "cli.usage=quanta-gqlang-v2-cli <query|fmt|v3-query|v3-fmt> (--file <path> | <input...>)\n",
    "cli.read_only=true\n",
    "cli.command_plane=forbidden"
);

#[derive(Copy, Clone)]
enum CliCommandV1 {
    Query,
    Fmt,
}

impl CliCommandV1 {
    fn parse_v1(raw: &str) -> Result<Self, CliErrorV1> {
        match raw {
            "query" | "v3-query" => Ok(Self::Query),
            "fmt" | "v3-fmt" => Ok(Self::Fmt),
            _ => Err(CliErrorV1::usage_v1(format!("unsupported command `{raw}`"))),
        }
    }
}

struct CliInvocationV1 {
    command: CliCommandV1,
    input: String,
}

struct CliOutputV1 {
    stdout: String,
    exit_code: i32,
}

enum CliErrorClassV1 {
    Usage,
    Io,
    Command,
}

struct CliErrorV1 {
    class: CliErrorClassV1,
    message: String,
}

impl CliErrorV1 {
    fn usage_v1(message: String) -> Self {
        Self {
            class: CliErrorClassV1::Usage,
            message,
        }
    }

    fn io_v1(message: String) -> Self {
        Self {
            class: CliErrorClassV1::Io,
            message,
        }
    }

    fn command_v1(message: String) -> Self {
        Self {
            class: CliErrorClassV1::Command,
            message,
        }
    }

    fn exit_code_v1(&self) -> i32 {
        match self.class {
            CliErrorClassV1::Usage => 2,
            CliErrorClassV1::Io | CliErrorClassV1::Command => 1,
        }
    }

    fn render_v1(&self) -> String {
        match self.class {
            CliErrorClassV1::Usage => format!(
                "cli.error.class=usage\ncli.error.message={}\n{}",
                self.message, USAGE_V1
            ),
            CliErrorClassV1::Io => {
                format!("cli.error.class=io\ncli.error.message={}", self.message)
            }
            CliErrorClassV1::Command => format!("cli.error.class=command\n{}", self.message),
        }
    }
}

fn main() {
    process::exit(run_cli_from_env_v1());
}

fn run_cli_from_env_v1() -> i32 {
    let args = env::args_os().skip(1).collect::<Vec<_>>();

    if args.len() == 1 && args.first().is_some_and(is_help_v1) {
        println!("{USAGE_V1}");
        return 0;
    }

    let invocation = match parse_invocation_v1(args) {
        Ok(invocation) => invocation,
        Err(err) => {
            eprintln!("{}", err.render_v1());
            return err.exit_code_v1();
        }
    };

    match execute_invocation_v1(invocation) {
        Ok(output) => {
            println!("{}", output.stdout);
            output.exit_code
        }
        Err(err) => {
            eprintln!("{}", err.render_v1());
            err.exit_code_v1()
        }
    }
}

fn is_help_v1(arg: &OsString) -> bool {
    matches!(arg.to_str(), Some("help" | "--help"))
}

fn parse_invocation_v1(args: Vec<OsString>) -> Result<CliInvocationV1, CliErrorV1> {
    let (command_raw, input_args) = args
        .split_first()
        .ok_or_else(|| CliErrorV1::usage_v1("missing command".to_string()))?;
    let command = CliCommandV1::parse_v1(
        command_raw
            .to_str()
            .ok_or_else(|| CliErrorV1::usage_v1("command must be valid UTF-8".to_string()))?,
    )?;

    let input = match input_args {
        [] => {
            return Err(CliErrorV1::usage_v1(
                "missing input or --file <path>".to_string(),
            ));
        }
        [flag] if is_help_v1(flag) => {
            return Err(CliErrorV1::usage_v1(
                "missing input or --file <path>".to_string(),
            ));
        }
        [flag, path] if flag == "--file" => read_input_file_v1(PathBuf::from(path))?,
        [flag, ..] if flag == "--file" => {
            return Err(CliErrorV1::usage_v1(
                "--file requires exactly one path argument".to_string(),
            ));
        }
        inline => inline_input_v1(inline)?,
    };

    if input.trim().is_empty() {
        return Err(CliErrorV1::usage_v1("input cannot be empty".to_string()));
    }

    Ok(CliInvocationV1 { command, input })
}

fn read_input_file_v1(path: PathBuf) -> Result<String, CliErrorV1> {
    fs::read_to_string(&path)
        .map_err(|err| CliErrorV1::io_v1(format!("failed to read {}: {err}", path.display())))
}

fn inline_input_v1(args: &[OsString]) -> Result<String, CliErrorV1> {
    let mut parts = Vec::with_capacity(args.len());
    for arg in args {
        let value = arg.clone().into_string().map_err(|raw| {
            CliErrorV1::usage_v1(format!(
                "input must be valid UTF-8, got `{}`",
                raw.to_string_lossy()
            ))
        })?;
        parts.push(value);
    }
    Ok(parts.join(" "))
}

fn execute_invocation_v1(invocation: CliInvocationV1) -> Result<CliOutputV1, CliErrorV1> {
    match invocation.command {
        CliCommandV1::Query => commands::v3_query::run(&invocation.input)
            .map(|stdout| CliOutputV1 {
                stdout,
                exit_code: 0,
            })
            .map_err(CliErrorV1::command_v1),
        CliCommandV1::Fmt => commands::v3_format::run(&invocation.input)
            .map(|stdout| CliOutputV1 {
                stdout,
                exit_code: 0,
            })
            .map_err(CliErrorV1::command_v1),
    }
}

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

use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::process;

use quanta_gqlang_v2_cli::commands;
use quanta_gqlang_v2_cli::doctor::{self, DoctorStatusV1};

const USAGE_V1: &str = concat!(
    "cli.usage=quanta-gqlang-v2-cli <parse|sema|query|fmt|explain|doctor|headless> (--file <path> | <input...>)\n",
    "cli.read_only=true\n",
    "cli.command_plane=forbidden"
);

#[derive(Copy, Clone)]
enum CliCommandV1 {
    Parse,
    Sema,
    Query,
    Fmt,
    Explain,
    Doctor,
    Headless,
}

impl CliCommandV1 {
    fn parse_v1(raw: &str) -> Result<Self, CliErrorV1> {
        match raw {
            "parse" => Ok(Self::Parse),
            "sema" => Ok(Self::Sema),
            "query" => Ok(Self::Query),
            "fmt" => Ok(Self::Fmt),
            "explain" => Ok(Self::Explain),
            "doctor" => Ok(Self::Doctor),
            "headless" => Ok(Self::Headless),
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
        let value = arg
            .clone()
            .into_string()
            .map_err(|_| CliErrorV1::usage_v1("input must be valid UTF-8".to_string()))?;
        parts.push(value);
    }
    Ok(parts.join(" "))
}

fn execute_invocation_v1(invocation: CliInvocationV1) -> Result<CliOutputV1, CliErrorV1> {
    match invocation.command {
        CliCommandV1::Parse => commands::parse::run(&invocation.input)
            .map(|report| CliOutputV1 {
                stdout: report.render_v1(),
                exit_code: 0,
            })
            .map_err(CliErrorV1::command_v1),
        CliCommandV1::Sema => commands::sema::run(&invocation.input)
            .map(|report| CliOutputV1 {
                stdout: report.render_v1(),
                exit_code: 0,
            })
            .map_err(CliErrorV1::command_v1),
        CliCommandV1::Query => commands::query::run(&invocation.input)
            .map(|report| CliOutputV1 {
                stdout: report.render_v1(),
                exit_code: 0,
            })
            .map_err(CliErrorV1::command_v1),
        CliCommandV1::Fmt => commands::format::run(&invocation.input)
            .map(|formatted| CliOutputV1 {
                stdout: formatted,
                exit_code: 0,
            })
            .map_err(CliErrorV1::command_v1),
        CliCommandV1::Explain => commands::explain::run(&invocation.input)
            .map(|view| CliOutputV1 {
                stdout: view.render_v1(),
                exit_code: 0,
            })
            .map_err(CliErrorV1::command_v1),
        CliCommandV1::Doctor => doctor::run_v1(&invocation.input)
            .map(|report| CliOutputV1 {
                stdout: report.render_v1(),
                exit_code: match report.status {
                    DoctorStatusV1::NoResult => 0,
                    DoctorStatusV1::ParseError | DoctorStatusV1::SemaError => 1,
                },
            })
            .map_err(CliErrorV1::command_v1),
        CliCommandV1::Headless => commands::headless::run(&invocation.input)
            .map(|report| CliOutputV1 {
                stdout: report.render_v1(),
                exit_code: 0,
            })
            .map_err(CliErrorV1::command_v1),
    }
}

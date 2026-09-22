//! Bounded read-only QGLang CLI.

use std::env;
use std::ffi::OsString;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::process::ExitCode;

use quanta_gqlang_v2_cli::commands;
use quanta_gqlang_v2_frontend_contract::FrontendBudgetV2;

const USAGE_V2: &str = concat!(
    "cli.usage=quanta-gqlang-v2-cli <query|fmt|v3-query|v3-fmt> (--file <path> | <input...>)\n",
    "cli.read_only=true\n",
    "cli.command_plane=forbidden"
);

#[derive(Copy, Clone)]
enum CliCommandV2 {
    Query,
    Fmt,
}

struct CliInvocationV2 {
    command: CliCommandV2,
    input: String,
}

struct CliErrorV2 {
    class: &'static str,
    message: String,
    exit_code: u8,
}

impl CliErrorV2 {
    fn usage(message: impl Into<String>) -> Self {
        Self {
            class: "usage",
            message: message.into(),
            exit_code: 2,
        }
    }

    fn io(message: impl Into<String>) -> Self {
        Self {
            class: "io",
            message: message.into(),
            exit_code: 1,
        }
    }

    fn render(&self) -> String {
        let mut rendered = format!(
            "cli.error.class={}\ncli.error.message={}",
            self.class, self.message
        );
        if self.class == "usage" {
            rendered.push('\n');
            rendered.push_str(USAGE_V2);
        }
        rendered
    }
}

fn main() -> ExitCode {
    ExitCode::from(run_cli())
}

fn run_cli() -> u8 {
    let mut probe = env::args_os().skip(1);
    if matches!(probe.size_hint(), (1, Some(1))) && probe.next().as_ref().is_some_and(is_help) {
        println!("{USAGE_V2}");
        return 0;
    }
    let budget = match FrontendBudgetV2::standard_v2() {
        Ok(budget) => budget,
        Err(error) => {
            eprintln!(
                "cli.error.class=configuration\ncli.error.message={}",
                error.message()
            );
            return 1;
        }
    };
    let invocation = match parse_invocation(env::args_os().skip(1), budget.max_source_bytes()) {
        Ok(invocation) => invocation,
        Err(error) => {
            eprintln!("{}", error.render());
            return error.exit_code;
        }
    };
    match invocation.command {
        CliCommandV2::Query => match commands::v3_query::run(&invocation.input) {
            Ok(output) => {
                println!("{}", output.body);
                output.exit_code
            }
            Err(error) => {
                eprintln!("cli.error.class=command\n{error}");
                1
            }
        },
        CliCommandV2::Fmt => match commands::v3_format::run(&invocation.input) {
            Ok(output) => {
                println!("{output}");
                0
            }
            Err(error) => {
                eprintln!("cli.error.class=command\n{error}");
                1
            }
        },
    }
}

fn parse_invocation(
    mut args: impl Iterator<Item = OsString>,
    max_source_bytes: u32,
) -> Result<CliInvocationV2, CliErrorV2> {
    let command = match args
        .next()
        .and_then(|arg| arg.into_string().ok())
        .as_deref()
    {
        Some("query" | "v3-query") => CliCommandV2::Query,
        Some("fmt" | "v3-fmt") => CliCommandV2::Fmt,
        Some(other) => return Err(CliErrorV2::usage(format!("unsupported command `{other}`"))),
        None => return Err(CliErrorV2::usage("missing command")),
    };
    let first = args
        .next()
        .ok_or_else(|| CliErrorV2::usage("missing input or --file <path>"))?;
    let input = if first == "--file" {
        let path = args
            .next()
            .ok_or_else(|| CliErrorV2::usage("--file requires exactly one path argument"))?;
        if args.next().is_some() {
            return Err(CliErrorV2::usage(
                "--file requires exactly one path argument",
            ));
        }
        read_input_file(PathBuf::from(path), max_source_bytes)?
    } else {
        inline_input(first, args, max_source_bytes)?
    };
    if input.trim().is_empty() {
        return Err(CliErrorV2::usage("input cannot be empty"));
    }
    Ok(CliInvocationV2 { command, input })
}

fn read_input_file(path: PathBuf, max_source_bytes: u32) -> Result<String, CliErrorV2> {
    let file = File::open(&path)
        .map_err(|error| CliErrorV2::io(format!("failed to open {}: {error}", path.display())))?;
    let limit = u64::from(max_source_bytes) + 1;
    let mut bytes = Vec::new();
    file.take(limit)
        .read_to_end(&mut bytes)
        .map_err(|error| CliErrorV2::io(format!("failed to read {}: {error}", path.display())))?;
    if bytes.len() > usize::try_from(max_source_bytes).unwrap_or(usize::MAX) {
        return Err(CliErrorV2::usage("input exceeds max_source_bytes"));
    }
    String::from_utf8(bytes)
        .map_err(|_invalid_utf8| CliErrorV2::usage("input file must be valid UTF-8"))
}

fn inline_input(
    first: OsString,
    rest: impl Iterator<Item = OsString>,
    max_source_bytes: u32,
) -> Result<String, CliErrorV2> {
    let limit = usize::try_from(max_source_bytes).unwrap_or(usize::MAX);
    let mut output = String::new();
    for raw in std::iter::once(first).chain(rest) {
        let part = raw
            .into_string()
            .map_err(|_invalid_utf8| CliErrorV2::usage("input must be valid UTF-8"))?;
        let separator = usize::from(!output.is_empty());
        if output
            .len()
            .saturating_add(separator)
            .saturating_add(part.len())
            > limit
        {
            return Err(CliErrorV2::usage("input exceeds max_source_bytes"));
        }
        if separator == 1 {
            output.push(' ');
        }
        output.push_str(&part);
    }
    Ok(output)
}

fn is_help(arg: &OsString) -> bool {
    matches!(arg.to_str(), Some("help" | "--help"))
}

//! Bounded read-only QGLang CLI.

use std::env;
use std::ffi::OsString;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use quanta_gqlang_v2_cli::line_value::encode_line_value_v1;
use quanta_gqlang_v2_cli::CliCommandKindV2;
use quanta_gqlang_v2_cli::{cli_command_alternation_v1, cli_command_kind_v1, commands};
use quanta_gqlang_v2_frontend_contract::FrontendBudgetV2;

/// Render the stable usage block from the single CLI command authority, so
/// the emitted alternation is the dispatched token set verbatim.
fn usage_v2() -> String {
    format!(
        "cli.usage=quanta-gqlang-v2-cli {} (--file <path> | <input...>)\n\
         cli.read_only=true\n\
         cli.command_plane=forbidden",
        cli_command_alternation_v1()
    )
}

#[derive(Debug)]
struct CliInvocationV2 {
    command: CliCommandKindV2,
    input: String,
}

#[derive(Debug)]
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
            self.class,
            encode_line_value_v1(&self.message)
        );
        if self.class == "usage" {
            rendered.push('\n');
            rendered.push_str(&usage_v2());
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
        println!("{}", usage_v2());
        return 0;
    }
    let budget = match FrontendBudgetV2::standard_v2() {
        Ok(budget) => budget,
        Err(error) => {
            eprintln!(
                "cli.error.class=configuration\ncli.error.message={}",
                encode_line_value_v1(error.message())
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
        CliCommandKindV2::Query => match commands::v3_query::run(&invocation.input) {
            Ok(output) => {
                println!("{}", output.body);
                output.exit_code
            }
            Err(error) => {
                eprintln!("cli.error.class=command\n{error}");
                1
            }
        },
        CliCommandKindV2::Fmt => match commands::v3_format::run(&invocation.input) {
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
    // CLI-01: dispatch resolves through the single library spec table. No
    // second handwritten token match may exist beside `CLI_COMMAND_SPECS_V1`.
    let raw = args.next();
    let command = match raw
        .as_ref()
        .and_then(|arg| arg.to_str())
        .and_then(cli_command_kind_v1)
    {
        Some(kind) => kind,
        None => {
            return Err(match raw.and_then(|arg| arg.into_string().ok()) {
                Some(other) => CliErrorV2::usage(format!("unsupported command `{other}`")),
                None => CliErrorV2::usage("missing command"),
            });
        }
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
    let file = open_input_file(&path)
        .map_err(|error| CliErrorV2::io(format!("failed to open {}: {error}", path.display())))?;
    let file_type = file.metadata().map_err(|error| {
        CliErrorV2::io(format!("failed to inspect {}: {error}", path.display()))
    })?;
    if !file_type.is_file() {
        return Err(CliErrorV2::io(format!(
            "input path is not a regular file: {}",
            path.display()
        )));
    }
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

#[cfg(unix)]
fn open_input_file(path: &Path) -> std::io::Result<File> {
    use std::fs::OpenOptions;
    use std::os::unix::fs::OpenOptionsExt;

    // Opening a FIFO for reading must not wait for a writer. File type is
    // checked on this descriptor, not on a separately inspected path.
    OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NONBLOCK)
        .open(path)
}

#[cfg(not(unix))]
fn open_input_file(path: &Path) -> std::io::Result<File> {
    File::open(path)
}

fn inline_input(
    first: OsString,
    rest: impl Iterator<Item = OsString>,
    max_source_bytes: u32,
) -> Result<String, CliErrorV2> {
    let limit = usize::try_from(max_source_bytes).unwrap_or(usize::MAX);
    let mut output = String::new();
    for (index, raw) in std::iter::once(first).chain(rest).enumerate() {
        let part = raw
            .into_string()
            .map_err(|_invalid_utf8| CliErrorV2::usage("input must be valid UTF-8"))?;
        let separator = usize::from(index > 0);
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

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "private parser unit tests assert exact success and error results"
)]
mod tests {
    use super::*;

    #[test]
    fn inline_budget_counts_inserted_argument_separators() {
        let exact = parse_invocation(
            [
                OsString::from("query"),
                OsString::from("ab"),
                OsString::from("cd"),
            ]
            .into_iter(),
            5,
        )
        .expect("two arguments plus separator fit exactly");
        assert_eq!(exact.input, "ab cd");

        let error = parse_invocation(
            [
                OsString::from("query"),
                OsString::from("ab"),
                OsString::from("cd"),
            ]
            .into_iter(),
            4,
        )
        .expect_err("inserted separator exceeds the limit");
        assert_eq!(error.class, "usage");
        assert_eq!(error.message, "input exceeds max_source_bytes");
        assert_eq!(error.exit_code, 2);
    }

    #[test]
    fn leading_empty_inline_argument_preserves_its_separator_and_budget() {
        let exact = parse_invocation(
            [
                OsString::from("query"),
                OsString::from(""),
                OsString::from("x"),
            ]
            .into_iter(),
            2,
        )
        .expect("empty first argument and separator fit exactly");
        assert_eq!(exact.input, " x");

        let error = parse_invocation(
            [
                OsString::from("query"),
                OsString::from(""),
                OsString::from("x"),
            ]
            .into_iter(),
            1,
        )
        .expect_err("separator exceeds one-byte source budget");
        assert_eq!(error.message, "input exceeds max_source_bytes");

        let repeated = parse_invocation(
            [
                OsString::from("query"),
                OsString::from(""),
                OsString::from(""),
                OsString::from("x"),
            ]
            .into_iter(),
            3,
        )
        .expect("two leading empty arguments retain both separators");
        assert_eq!(repeated.input, "  x");
    }

    #[cfg(unix)]
    #[test]
    fn invalid_utf8_inline_argument_is_rejected_before_analysis() {
        use std::os::unix::ffi::OsStringExt;

        let error = parse_invocation(
            [OsString::from("query"), OsString::from_vec(vec![0xff])].into_iter(),
            32,
        )
        .expect_err("invalid UTF-8 argv");
        assert_eq!(error.class, "usage");
        assert_eq!(error.message, "input must be valid UTF-8");
        assert_eq!(error.exit_code, 2);
    }
}

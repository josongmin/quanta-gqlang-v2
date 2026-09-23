//! Bounded read-only NDJSON diagnostics transport.

use std::io::{Read, Write};

pub mod handlers;
pub mod server;

use server::{DiagnosticsServerV2, ProtocolErrorV2};

/// Run one NDJSON session. Each physical line is one request; a malformed or
/// oversized line yields one error and does not poison the following line.
pub fn run_stdio_session_v2<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
) -> Result<u8, std::io::Error> {
    let server = DiagnosticsServerV2::try_standard().map_err(protocol_configuration_error)?;
    run_with_server(reader, writer, &server)
}

/// Test/embedding entry with an explicit checked budget.
pub fn run_stdio_session_with_budget_v2<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    budget: quanta_gqlang_v2_frontend_contract::FrontendBudgetV2,
) -> Result<u8, std::io::Error> {
    let server =
        DiagnosticsServerV2::try_with_budget(budget).map_err(protocol_configuration_error)?;
    run_with_server(reader, writer, &server)
}

fn protocol_configuration_error(error: ProtocolErrorV2) -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        format!("{}: {}", error.code, error.message),
    )
}

fn run_with_server<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    server: &DiagnosticsServerV2,
) -> Result<u8, std::io::Error> {
    let frame_limit = usize::try_from(server.budget().max_frame_bytes()).unwrap_or(usize::MAX);
    let mut line = Vec::new();
    let mut overflow = false;
    let mut status = 0_u8;
    let mut chunk = [0_u8; 4096];
    loop {
        let read = match reader.read(&mut chunk) {
            Ok(read) => read,
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        };
        if read == 0 {
            break;
        }
        for byte in chunk.get(..read).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "reader returned invalid length",
            )
        })? {
            if *byte == b'\n' {
                process_line(server, &line, overflow, writer, &mut status)?;
                line.clear();
                overflow = false;
            } else if !overflow {
                if line.len() == frame_limit {
                    overflow = true;
                    line.clear();
                } else {
                    line.push(*byte);
                }
            }
        }
    }
    if overflow || !line.is_empty() {
        process_line(server, &line, overflow, writer, &mut status)?;
    }
    Ok(status)
}

#[must_use]
pub fn run_stdio_front_door_v2() -> u8 {
    let mut stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout().lock();
    match run_stdio_session_v2(&mut stdin, &mut stdout) {
        Ok(status) => status,
        Err(error) => {
            let payload = format!(
                "{{\"id\":null,\"ok\":false,\"error\":{{\"code\":\"io_error\",\"message\":{}}}}}",
                serde_json::to_string(&error.to_string())
                    .unwrap_or_else(|_| "\"stdio failure\"".to_owned())
            );
            let _ignored = writeln!(std::io::stderr().lock(), "{payload}");
            1
        }
    }
}

fn process_line<W: Write>(
    server: &DiagnosticsServerV2,
    line: &[u8],
    overflow: bool,
    writer: &mut W,
    status: &mut u8,
) -> Result<(), std::io::Error> {
    if !overflow && line.iter().all(u8::is_ascii_whitespace) {
        return Ok(());
    }
    let payload = if overflow {
        *status = 1;
        server.render_transport_error(ProtocolErrorV2::new(
            "frame_limit_exceeded",
            format!(
                "request exceeds max_frame_bytes {}",
                server.budget().max_frame_bytes()
            ),
        ))
    } else {
        match server.dispatch(line) {
            Ok(payload) => payload,
            Err(error) => {
                *status = 1;
                server.render_transport_error(error)
            }
        }
    };
    writer.write_all(&payload)?;
    writer.write_all(b"\n")?;
    // One request/response exchange must make progress before the reader asks
    // for another frame. Without an explicit flush, a generic buffered writer
    // can retain the response while a client waits for it, deadlocking a
    // long-lived NDJSON session.
    writer.flush()
}

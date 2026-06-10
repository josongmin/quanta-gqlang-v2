use std::io::{Read, Write};

pub mod handlers;
pub(crate) mod json;
pub mod server;

pub fn run_stdio_session_v1<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
) -> Result<u8, std::io::Error> {
    let server = server::LspServerV1::new();
    let mut status = 0_u8;
    let mut frame_buffer = Vec::new();
    let mut read_buffer = [0_u8; 4096];

    loop {
        drain_request_frames_v1(&server, &mut frame_buffer, writer, false, &mut status)?;
        let read = reader.read(&mut read_buffer)?;
        if read == 0 {
            break;
        }
        let read_slice = read_buffer.get(..read).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "reader reported a byte count beyond the read buffer length",
            )
        })?;
        frame_buffer.extend_from_slice(read_slice);
    }

    drain_request_frames_v1(&server, &mut frame_buffer, writer, true, &mut status)?;
    Ok(status)
}

#[must_use]
pub fn run_stdio_front_door_v1() -> u8 {
    let mut stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout().lock();
    match run_stdio_session_v1(&mut stdin, &mut stdout) {
        Ok(status) => status,
        Err(err) => {
            #[expect(
                clippy::let_underscore_must_use,
                reason = "stderr is the last-resort diagnostics channel; a failed write here has no further channel to report to, and the nonzero exit status below still signals the failure"
            )]
            let _ = writeln!(
                std::io::stderr().lock(),
                "{}",
                server::StdioLspErrorV1::io_failure_v1("stdio", err.to_string()).render_json_v1()
            );
            1
        }
    }
}

fn drain_request_frames_v1<W: Write>(
    server: &server::LspServerV1,
    frame_buffer: &mut Vec<u8>,
    writer: &mut W,
    eof: bool,
    status: &mut u8,
) -> Result<(), std::io::Error> {
    loop {
        match server::extract_next_request_frame_v1(frame_buffer, eof) {
            Ok(Some((start, end))) => {
                let request_bytes = frame_buffer.get(start..end).ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "request frame bounds exceeded the buffered stdio payload",
                    )
                })?;
                let payload = match std::str::from_utf8(request_bytes) {
                    Ok(request) => match server.dispatch_json_v1(request) {
                        Ok(output) => output,
                        Err(err) => {
                            *status = 1;
                            err.render_json_v1()
                        }
                    },
                    Err(err) => {
                        *status = 1;
                        server::StdioLspErrorV1::new(
                            "malformed_json",
                            format!("request bytes are not valid UTF-8: {err}"),
                        )
                        .render_json_v1()
                    }
                };
                writer.write_all(payload.as_bytes())?;
                writer.write_all(b"\n")?;
                frame_buffer.drain(..end);
            }
            Ok(None) => {
                if eof && !frame_buffer.iter().all(u8::is_ascii_whitespace) {
                    *status = 1;
                    let payload = server::StdioLspErrorV1::new(
                        "malformed_json",
                        "incomplete JSON object at end of input",
                    )
                    .render_json_v1();
                    writer.write_all(payload.as_bytes())?;
                    writer.write_all(b"\n")?;
                    frame_buffer.clear();
                }
                return Ok(());
            }
            Err(err) => {
                *status = 1;
                writer.write_all(err.render_json_v1().as_bytes())?;
                writer.write_all(b"\n")?;
                frame_buffer.clear();
                return Ok(());
            }
        }
    }
}

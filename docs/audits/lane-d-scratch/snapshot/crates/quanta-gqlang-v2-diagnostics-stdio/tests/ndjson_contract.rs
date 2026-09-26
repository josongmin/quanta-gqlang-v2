#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "wire contract tests inspect exact response positions"
)]

use std::io::{self, Cursor, Read, Write};

use quanta_gqlang_v2_diagnostics_stdio::run_stdio_session_v2;
use serde_json::Value;

fn run(bytes: &[u8]) -> (u8, Vec<Value>) {
    let mut input = Cursor::new(bytes);
    let mut output = Vec::new();
    let status = run_stdio_session_v2(&mut input, &mut output).expect("stdio session");
    let values = output
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| serde_json::from_slice(line).expect("response json"))
        .collect();
    (status, values)
}

fn valid(id: u32) -> String {
    format!(r#"{{"id":{id},"method":"v3_diagnostics","source_id":7,"source":"show capabilities"}}"#)
}

struct ScriptedReader {
    bytes: Vec<u8>,
    cursor: usize,
    chunks: Vec<usize>,
    reads: usize,
    fail_at: Option<usize>,
}

impl ScriptedReader {
    fn new(bytes: &[u8], chunks: Vec<usize>) -> Self {
        Self {
            bytes: bytes.to_vec(),
            cursor: 0,
            chunks,
            reads: 0,
            fail_at: None,
        }
    }

    fn failing(bytes: &[u8], fail_at: usize) -> Self {
        Self {
            bytes: bytes.to_vec(),
            cursor: 0,
            chunks: vec![1],
            reads: 0,
            fail_at: Some(fail_at),
        }
    }
}

impl Read for ScriptedReader {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if self.fail_at == Some(self.reads) {
            return Err(io::Error::other("scripted read failure"));
        }
        if self.cursor == self.bytes.len() {
            return Ok(0);
        }
        let requested = self.chunks[self.reads % self.chunks.len()];
        self.reads += 1;
        let count = requested
            .min(buffer.len())
            .min(self.bytes.len() - self.cursor);
        buffer[..count].copy_from_slice(&self.bytes[self.cursor..self.cursor + count]);
        self.cursor += count;
        Ok(count)
    }
}

struct InvalidLengthReader;

impl Read for InvalidLengthReader {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        Ok(buffer.len() + 1)
    }
}

struct InterruptingReader<'a> {
    inner: Cursor<&'a [u8]>,
    interrupt_at: u64,
    interrupted: bool,
}

impl Read for InterruptingReader<'_> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if self.inner.position() == self.interrupt_at && !self.interrupted {
            self.interrupted = true;
            return Err(io::ErrorKind::Interrupted.into());
        }
        let limit = if self.inner.position() == 0 {
            usize::try_from(self.interrupt_at)
                .expect("test request fits usize")
                .min(output.len())
        } else {
            output.len()
        };
        self.inner.read(&mut output[..limit])
    }
}

struct ShortWriter {
    bytes: Vec<u8>,
    interrupt: bool,
    zero: bool,
    flushes: usize,
}

impl Write for ShortWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        if self.interrupt {
            self.interrupt = false;
            return Err(io::ErrorKind::Interrupted.into());
        }
        if self.zero {
            return Ok(0);
        }
        let count = buffer.len().min(3);
        self.bytes.extend_from_slice(&buffer[..count]);
        Ok(count)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.flushes += 1;
        Ok(())
    }
}

#[test]
fn short_and_interrupted_writes_preserve_exact_response_and_framing() {
    const REQUEST: &[u8] = b"{\"id\":17,\"method\":\"shutdown\",\"source_id\":1,\"source\":\"\"}\n";
    const EXPECTED: &[u8] = b"{\"id\":17,\"ok\":false,\"error\":{\"code\":\"method_not_allowed\",\"message\":\"method `shutdown` is not allowed\"}}\n";
    for interrupt in [false, true] {
        let mut input = Cursor::new(REQUEST);
        let mut writer = ShortWriter {
            bytes: Vec::new(),
            interrupt,
            zero: false,
            flushes: 0,
        };
        let status = run_stdio_session_v2(&mut input, &mut writer).expect("short-write session");
        assert_eq!(
            status, 0,
            "serialized method rejection is not a transport failure"
        );
        assert_eq!(
            writer.bytes, EXPECTED,
            "every payload and delimiter byte must be written"
        );
        assert_eq!(writer.flushes, 1);
    }
    let mut input = Cursor::new(REQUEST);
    let mut writer = ShortWriter {
        bytes: Vec::new(),
        interrupt: false,
        zero: true,
        flushes: 0,
    };
    let error = run_stdio_session_v2(&mut input, &mut writer)
        .expect_err("zero write cannot fabricate success");
    assert_eq!(error.kind(), io::ErrorKind::WriteZero);
    assert!(writer.bytes.is_empty());
    assert_eq!(writer.flushes, 0);
}

struct FailingWriter {
    calls: usize,
    fail_at: usize,
    fail_flush: bool,
    flushes: usize,
}

impl Write for FailingWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        if self.calls == self.fail_at {
            return Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "scripted write failure",
            ));
        }
        self.calls += 1;
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.flushes += 1;
        if self.fail_flush {
            return Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "scripted flush failure",
            ));
        }
        Ok(())
    }
}

#[test]
fn valid_request_echoes_id_and_reports_honest_incomplete_analysis() {
    let request = format!("{}\n", valid(41));
    let (status, responses) = run(request.as_bytes());
    assert_eq!(status, 0);
    assert_eq!(responses.len(), 1);
    let mut response_keys: Vec<&str> = responses[0]
        .as_object()
        .expect("success response is an object")
        .keys()
        .map(String::as_str)
        .collect();
    response_keys.sort_unstable();
    assert_eq!(
        response_keys,
        ["analysis_status", "diagnostics", "id", "method", "ok"]
    );
    assert_eq!(responses[0]["id"], 41);
    assert_eq!(responses[0]["ok"], true);
    assert_eq!(responses[0]["method"], "v3_diagnostics");
    assert_eq!(responses[0]["analysis_status"], "incomplete");
    let diagnostics = responses[0]["diagnostics"]
        .as_array()
        .expect("diagnostics array");
    assert_eq!(
        diagnostics.len(),
        3,
        "unexpected diagnostics: {}",
        responses[0]
    );
    let codes: Vec<&str> = diagnostics
        .iter()
        .map(|diagnostic| diagnostic["code"].as_str().expect("diagnostic code"))
        .collect();
    assert_eq!(
        codes,
        [
            "pipeline.analysis.missing_graph_authority",
            "pipeline.analysis.missing_module_authority",
            "pipeline.analysis.missing_relation_authority",
        ]
    );
    for diagnostic in diagnostics {
        let mut keys: Vec<&str> = diagnostic
            .as_object()
            .expect("diagnostic is an object")
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            [
                "code",
                "message",
                "severity",
                "source_id",
                "span_end",
                "span_start",
                "stage"
            ]
        );
        assert_eq!(diagnostic["stage"], "declaration");
        assert_eq!(diagnostic["severity"], "warning");
        assert_eq!(diagnostic["source_id"], 7);
        assert_eq!(diagnostic["span_start"], 0);
        assert_eq!(diagnostic["span_end"], 17);
        assert!(
            diagnostic["message"]
                .as_str()
                .is_some_and(|message| !message.is_empty()),
            "unexpected diagnostic: {diagnostic}"
        );
    }
}

#[test]
fn malformed_invalid_utf8_and_duplicate_lines_do_not_poison_following_requests() {
    let mut stream = Vec::new();
    stream.extend_from_slice(b"{not json}\n");
    stream.extend_from_slice(b"\xff\n");
    stream.extend_from_slice(
        br#"{"id":1,"id":2,"method":"v3_diagnostics","source_id":1,"source":"x"}"#,
    );
    stream.push(b'\n');
    stream.extend_from_slice(valid(9).as_bytes());
    stream.push(b'\n');

    let (status, responses) = run(&stream);
    assert_eq!(status, 1);
    assert_eq!(responses.len(), 4);
    for response in &responses[..3] {
        let mut keys: Vec<&str> = response
            .as_object()
            .expect("error response is an object")
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();
        assert_eq!(keys, ["error", "id", "ok"]);
        let mut error_keys: Vec<&str> = response["error"]
            .as_object()
            .expect("protocol error is an object")
            .keys()
            .map(String::as_str)
            .collect();
        error_keys.sort_unstable();
        assert_eq!(error_keys, ["code", "message"]);
        assert_eq!(response["ok"], false);
        assert_eq!(response["id"], Value::Null);
        assert_eq!(response["error"]["code"], "invalid_request");
    }
    assert_eq!(responses[3]["id"], 9);
    assert_eq!(responses[3]["ok"], true);
}

#[test]
fn protocol_is_one_object_per_line_not_brace_balanced_framing() {
    let glued = format!("{}{}\n{}\n", valid(1), valid(2), valid(3));
    let (status, responses) = run(glued.as_bytes());
    assert_eq!(status, 1);
    assert_eq!(responses.len(), 2);
    assert_eq!(responses[0]["ok"], false);
    assert_eq!(responses[0]["error"]["code"], "invalid_request");
    assert_eq!(responses[1]["id"], 3);
    assert_eq!(responses[1]["ok"], true);
}

#[test]
fn response_is_byte_deterministic() {
    let request = format!("{}\n", valid(5));
    let mut first_input = Cursor::new(request.as_bytes());
    let mut first = Vec::new();
    let mut second_input = Cursor::new(request.as_bytes());
    let mut second = Vec::new();
    assert_eq!(
        run_stdio_session_v2(&mut first_input, &mut first).expect("first"),
        0
    );
    assert!(
        !first.is_empty(),
        "a valid request must emit at least one NDJSON response"
    );
    assert_eq!(
        run_stdio_session_v2(&mut second_input, &mut second).expect("second"),
        0
    );
    assert_eq!(first, second);
}

#[test]
fn arbitrary_short_reads_and_eof_without_newline_preserve_framing() {
    let unicode = r#"{"id":3,"method":"v3_diagnostics","source_id":7,"source":"let 이름 = 1"}"#;
    let stream = format!("{}\n{}\n{unicode}", valid(1), valid(2));
    let mut baseline_input = Cursor::new(stream.as_bytes());
    let mut baseline = Vec::new();
    let baseline_status =
        run_stdio_session_v2(&mut baseline_input, &mut baseline).expect("baseline session");
    assert!(
        !baseline.is_empty(),
        "the baseline session must emit responses before chunked comparison"
    );

    for chunks in [
        vec![1],
        vec![2, 1, 3],
        vec![4095],
        vec![4096],
        vec![stream.find('\n').expect("newline")],
        vec![stream.find('\n').expect("newline") + 1],
    ] {
        let mut input = ScriptedReader::new(stream.as_bytes(), chunks.clone());
        let mut output = Vec::new();
        let status = run_stdio_session_v2(&mut input, &mut output).expect("chunked session");
        assert_eq!(
            status, baseline_status,
            "status drift for chunks {chunks:?}"
        );
        assert_eq!(output, baseline, "response drift for chunks {chunks:?}");
    }
}

#[test]
fn interrupted_read_preserves_partial_frame_and_following_request() {
    let first = format!("{}\n", valid(1));
    let stream = format!("{first}{}\n", valid(2));
    for interrupt_at in [first.len() / 2, first.len()] {
        let mut input = InterruptingReader {
            inner: Cursor::new(stream.as_bytes()),
            interrupt_at: u64::try_from(interrupt_at).expect("test request fits u64"),
            interrupted: false,
        };
        let mut output = Vec::new();
        let status =
            run_stdio_session_v2(&mut input, &mut output).expect("interrupted read must retry");
        let responses: Vec<Value> = output
            .split(|byte| *byte == b'\n')
            .filter(|line| !line.is_empty())
            .map(|line| serde_json::from_slice(line).expect("response json"))
            .collect();
        assert_eq!(status, 0, "interrupt_at={interrupt_at}");
        assert_eq!(responses.len(), 2, "interrupt_at={interrupt_at}");
        assert_eq!(responses[0]["id"], 1);
        assert_eq!(responses[1]["id"], 2);
        assert!(input.interrupted, "reader must inject Interrupted");
    }
}

#[test]
fn whitespace_lines_split_across_reads_are_ignored() {
    let stream = format!(" \t\r\n\n{}\n   ", valid(7));
    let mut input = ScriptedReader::new(stream.as_bytes(), vec![1, 2]);
    let mut output = Vec::new();
    let status = run_stdio_session_v2(&mut input, &mut output).expect("whitespace session");
    assert_eq!(status, 0);
    let lines: Vec<&[u8]> = output
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .collect();
    assert_eq!(lines.len(), 1);
    let response: Value = serde_json::from_slice(lines[0]).expect("response json");
    assert_eq!(response["id"], 7);
    assert_eq!(response["ok"], true);
}

#[test]
fn reader_and_writer_failures_are_returned_without_being_reclassified() {
    let request = format!("{}\n", valid(1));
    let mut reader = ScriptedReader::failing(request.as_bytes(), 1);
    let mut output = Vec::new();
    let error = run_stdio_session_v2(&mut reader, &mut output).expect_err("read must fail");
    assert_eq!(error.kind(), io::ErrorKind::Other);

    for fail_at in [0, 1] {
        let mut input = Cursor::new(request.as_bytes());
        let mut writer = FailingWriter {
            calls: 0,
            fail_at,
            fail_flush: false,
            flushes: 0,
        };
        let error = run_stdio_session_v2(&mut input, &mut writer).expect_err("write must fail");
        assert_eq!(error.kind(), io::ErrorKind::BrokenPipe);
    }

    let mut input = Cursor::new(request.as_bytes());
    let mut writer = FailingWriter {
        calls: 0,
        fail_at: usize::MAX,
        fail_flush: true,
        flushes: 0,
    };
    let error = run_stdio_session_v2(&mut input, &mut writer).expect_err("flush must fail");
    assert_eq!(error.kind(), io::ErrorKind::BrokenPipe);
    assert_eq!(writer.flushes, 1);
}

#[test]
fn every_response_is_flushed_before_the_next_request() {
    let stream = format!("{}\n{}\n", valid(1), valid(2));
    let mut input = Cursor::new(stream.as_bytes());
    let mut writer = FailingWriter {
        calls: 0,
        fail_at: usize::MAX,
        fail_flush: false,
        flushes: 0,
    };
    let status = run_stdio_session_v2(&mut input, &mut writer).expect("flushed session");
    assert_eq!(status, 0);
    assert_eq!(writer.flushes, 2, "one flush per emitted NDJSON response");
}

#[test]
fn reader_reporting_more_than_the_supplied_buffer_fails_closed() {
    let mut reader = InvalidLengthReader;
    let mut output = Vec::new();
    let error = run_stdio_session_v2(&mut reader, &mut output).expect_err("invalid length");
    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert!(output.is_empty());
}

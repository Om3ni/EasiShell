//! Spawn an interpreter and stream its output.
//!
//! Design notes (the parts that bite if done naively):
//! - **stdout and stderr each get their own reader task** so neither OS pipe
//!   buffer can fill and deadlock the child.
//! - We read **raw byte chunks**, not lines: `BufReader::lines()` would hide a
//!   prompt written without a trailing newline and strip `\r`/ANSI.
//! - Bytes are decoded with a **stateful** `encoding_rs` decoder so a multibyte
//!   character split across two reads is not corrupted into U+FFFD.
//! - Events flow to a generic `mpsc::Sender`, not a Tauri type, so this core is
//!   reusable by both the in-process backend and the elevated broker, and is
//!   testable without Tauri.

use std::path::PathBuf;
use std::process::Stdio;

use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::process::Command;
use tokio::sync::mpsc;

use super::events::{RunEvent, RunId, RunStatus};

/// What to run: an interpreter path plus its argument vector. Arguments are
/// passed as discrete argv elements (never a joined string) — injection-safe.
#[derive(Debug, Clone)]
pub struct RunSpec {
    pub run_id: RunId,
    pub program: PathBuf,
    pub args: Vec<String>,
    /// Short interpreter label for the `Started` event (e.g. "pwsh").
    pub interp_label: String,
}

#[derive(Clone, Copy)]
enum Stream {
    Out,
    Err,
}

/// Spawn the process described by `spec` and stream its lifecycle to `events`.
///
/// Returns when the child has exited and both reader tasks have drained. The
/// final `RunEvent::Exit` is sent before returning.
pub async fn run(spec: RunSpec, events: mpsc::Sender<RunEvent>) -> std::io::Result<RunStatus> {
    let mut cmd = Command::new(&spec.program);
    cmd.args(&spec.args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    #[cfg(windows)]
    {
        // CREATE_NO_WINDOW: don't flash a console window for the child.
        // (`creation_flags` is an inherent method on tokio's Command.)
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let mut child = cmd.spawn()?;
    let pid = child.id().unwrap_or(0);

    let _ = events
        .send(RunEvent::Started {
            run_id: spec.run_id,
            interp: spec.program.display().to_string(),
            interp_version: String::new(), // populated by a version probe later
            pid,
        })
        .await;

    let stdout = child.stdout.take().expect("stdout was piped");
    let stderr = child.stderr.take().expect("stderr was piped");

    // Both pipes are drained continuously on independent tasks → no deadlock.
    let out_task = tokio::spawn(pump(stdout, Stream::Out, spec.run_id, events.clone()));
    let err_task = tokio::spawn(pump(stderr, Stream::Err, spec.run_id, events.clone()));

    // Readers hit EOF when the child's pipe ends close, so this resolves promptly.
    let _ = out_task.await;
    let _ = err_task.await;

    let exit = child.wait().await?;
    let status = if exit.success() {
        RunStatus::Completed
    } else {
        RunStatus::Failed
    };

    let _ = events
        .send(RunEvent::Exit {
            run_id: spec.run_id,
            code: exit.code(),
            status,
        })
        .await;

    Ok(status)
}

/// Drain one pipe, decoding bytes to UTF-8 incrementally and emitting chunks.
async fn pump<R: AsyncRead + Unpin>(
    mut reader: R,
    kind: Stream,
    run_id: RunId,
    events: mpsc::Sender<RunEvent>,
) {
    let mut decoder = encoding_rs::UTF_8.new_decoder();
    let mut buf = [0u8; 8192];
    // `decode_to_string` writes into the String's spare capacity and returns
    // `OutputFull` *without growing it* — so we must pre-reserve enough room
    // (clearing the String below retains this capacity).
    let cap = decoder
        .max_utf8_buffer_length(buf.len())
        .unwrap_or(buf.len() * 4);
    let mut text = String::with_capacity(cap);

    loop {
        match reader.read(&mut buf).await {
            Ok(0) => break, // EOF
            Ok(n) => {
                text.clear(); // retains reserved capacity
                // `last = false`: a trailing incomplete multibyte sequence stays
                // buffered in the decoder for the next read (no U+FFFD at the seam).
                let _ = decoder.decode_to_string(&buf[..n], &mut text, false);
                if text.is_empty() {
                    continue;
                }
                if emit(&events, kind, run_id, &text).await.is_err() {
                    break; // consumer gone; stop reading
                }
            }
            Err(_) => break,
        }
    }

    // Flush any bytes the decoder was holding.
    text.clear();
    let _ = decoder.decode_to_string(b"", &mut text, true);
    if !text.is_empty() {
        let _ = emit(&events, kind, run_id, &text).await;
    }
}

async fn emit(
    events: &mpsc::Sender<RunEvent>,
    kind: Stream,
    run_id: RunId,
    chunk: &str,
) -> Result<(), mpsc::error::SendError<RunEvent>> {
    let ev = match kind {
        Stream::Out => RunEvent::Stdout {
            run_id,
            chunk: chunk.to_owned(),
        },
        Stream::Err => RunEvent::Stderr {
            run_id,
            chunk: chunk.to_owned(),
        },
    };
    events.send(ev).await
}

#[cfg(test)]
mod tests {
    use super::super::interp;
    use super::*;

    /// The tracer bullet: actually spawn PowerShell and confirm we receive its
    /// output as a stream and a clean exit. Proves the riskiest assumption.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn streams_output_and_exits_cleanly() {
        let interp = interp::detect();
        let spec = RunSpec {
            run_id: 1,
            program: interp.path.clone(),
            args: vec![
                "-NoLogo".into(),
                "-NoProfile".into(),
                "-Command".into(),
                // Single quotes dodge command-line double-quote escaping; the real
                // runner uses `-File <temp.ps1>`, so this never applies in practice.
                "1..3 | ForEach-Object { 'line ' + $_ }".into(),
            ],
            interp_label: interp.label().into(),
        };

        let (tx, mut rx) = mpsc::channel(64);
        let handle = tokio::spawn(run(spec, tx));

        let mut stdout = String::new();
        let mut stderr = String::new();
        let mut exit = None;
        while let Some(ev) = rx.recv().await {
            match ev {
                RunEvent::Stdout { chunk, .. } => stdout.push_str(&chunk),
                RunEvent::Stderr { chunk, .. } => stderr.push_str(&chunk),
                RunEvent::Exit { status, code, .. } => exit = Some((status, code)),
                RunEvent::Started { .. } => {}
            }
        }

        let run_result = handle.await.expect("run task panicked");
        assert!(run_result.is_ok(), "run returned error: {run_result:?}");
        assert!(
            stdout.contains("line 1") && stdout.contains("line 3"),
            "expected streamed lines.\n  stdout: {stdout:?}\n  stderr: {stderr:?}"
        );
        assert_eq!(
            exit,
            Some((RunStatus::Completed, Some(0))),
            "stderr: {stderr:?}"
        );
    }
}

//! Spawn a PowerShell process and stream whatever it prints back out.
//!
//! A couple things in here are non-obvious and already bit me, so theyre worth
//! spelling out. stdout and stderr each get there own reader task: if we only
//! drained one, the other pipe's OS buffer could fill up and the child would just
//! block forever waiting on us. We also read raw byte chunks instead of lines, on
//! purpose — something like `Read-Host` writes its prompt with no trailing newline,
//! and a line reader would swallow it until the user already needed to be typing.
//!
//! Events leave over a plain mpsc channel, nothing Tauri-shaped. that's deliberate:
//! the same core then works for both the in-process backend and the elevated broker
//! we add later, and we can test it without standing up the whole app.

use std::path::PathBuf;
use std::process::Stdio;

use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::process::Command;
use tokio::sync::mpsc;

use super::events::{RunEvent, RunId, RunStatus};

/// What to run. The args stay as separate argv elements and are never glued into a
/// single string — that's the thing that stops a value containing spaces or quotes
/// from sneaking in extra commands.
#[derive(Debug, Clone)]
pub struct RunSpec {
    pub run_id: RunId,
    pub program: PathBuf,
    pub args: Vec<String>,
    /// short label for the Started event, e.g. "pwsh". purely cosmetic.
    pub interp_label: String,
}

#[derive(Clone, Copy)]
enum Stream {
    Out,
    Err,
}

/// Spawn the process and pump its whole lifecycle into `events`. Comes back only once
/// the child has exited *and* both readers have drained, with the final Exit sent.
pub async fn run(spec: RunSpec, events: mpsc::Sender<RunEvent>) -> std::io::Result<RunStatus> {
    let mut cmd = Command::new(&spec.program);
    cmd.args(&spec.args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    #[cfg(windows)]
    {
        // keep the child from flashing up its own console window. creation_flags is an
        // inherent method on tokio's Command so theres no trait to import for this.
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let mut child = cmd.spawn()?;
    let pid = child.id().unwrap_or(0);

    let _ = events
        .send(RunEvent::Started {
            run_id: spec.run_id,
            interp: spec.program.display().to_string(),
            interp_version: String::new(), // a version probe will fill this in later
            pid,
        })
        .await;

    let stdout = child.stdout.take().expect("stdout was piped");
    let stderr = child.stderr.take().expect("stderr was piped");

    // each pipe gets drained on its own task, at the same time — this is the bit that
    // avoids the deadlock described up top.
    let out_task = tokio::spawn(pump(stdout, Stream::Out, spec.run_id, events.clone()));
    let err_task = tokio::spawn(pump(stderr, Stream::Err, spec.run_id, events.clone()));

    // both readers stop at EOF, which the child triggers when it closes its pipes on
    // exit. so by the time we get here, wait() is basically instant.
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

async fn pump<R: AsyncRead + Unpin>(
    mut reader: R,
    kind: Stream,
    run_id: RunId,
    events: mpsc::Sender<RunEvent>,
) {
    let mut decoder = encoding_rs::UTF_8.new_decoder();
    let mut buf = [0u8; 8192];

    // heads up, this one's a trap: decode_to_string writes into the String's *spare
    // capacity* and just returns OutputFull instead of growing it. a freshly cleared
    // String can have zero spare room, so skip this reserve and every single decode
    // quietly produces nothing — which looks exactly like "the process printed nothing"
    // and sent me chasing the wrong thing for a while. clear() keeps the capacity, so
    // reserving once up here is enough.
    let cap = decoder
        .max_utf8_buffer_length(buf.len())
        .unwrap_or(buf.len() * 4);
    let mut text = String::with_capacity(cap);

    loop {
        match reader.read(&mut buf).await {
            Ok(0) => break, // EOF
            Ok(n) => {
                text.clear();
                // last = false keeps a trailing half-finished multibyte char buffered
                // in the decoder so it can join the next read, instead of getting
                // mangled into U+FFFD right at the seam.
                let _ = decoder.decode_to_string(&buf[..n], &mut text, false);
                if text.is_empty() {
                    continue;
                }
                if emit(&events, kind, run_id, &text).await.is_err() {
                    break; // receiver's gone, no point reading more
                }
            }
            Err(_) => break,
        }
    }

    // flush whatever the decoder was still holding onto
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

    // the tracer bullet. actually launches PowerShell and checks the output comes back
    // as a stream and the process exits clean. if this is green, the scariest part of
    // the whole app is proven.
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
                // single quotes here sidestep the command-line double-quote escaping mess.
                // the real runner uses `-File <temp.ps1>` so it never has to deal with this.
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
        assert_eq!(exit, Some((RunStatus::Completed, Some(0))), "stderr: {stderr:?}");
    }
}

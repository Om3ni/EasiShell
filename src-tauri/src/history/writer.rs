//! Turning a finished run into its markdown file.
//!
//! The `.md` is the actual record of a run, not the sqlite row (that comes later and is
//! just an index pointing here). metadata header up top so you can skim what happened,
//! then the raw captured output in a fenced block.

use std::fs;
use std::path::{Path, PathBuf};

use super::naming;

/// Everything we need to render one run. Borrowed strings — the caller already has all
/// this sitting around, no reason to make it hand over ownership.
pub struct RunReport<'a> {
    pub command: &'a str,
    pub interp: &'a str,
    pub started: &'a str,
    pub finished: &'a str,
    pub duration_ms: u128,
    pub exit_code: Option<i32>,
    pub status: &'a str,
    pub output: &'a str,
}

/// Write the report into `history_dir` and give back the path we used. Creates the
/// directory if its not there yet.
pub fn write_report(history_dir: &Path, report: &RunReport) -> std::io::Result<PathBuf> {
    fs::create_dir_all(history_dir)?;
    let name = naming::log_filename(|n| history_dir.join(n).exists());
    let path = history_dir.join(name);
    fs::write(&path, render(report))?;
    Ok(path)
}

fn render(r: &RunReport) -> String {
    let code = match r.exit_code {
        Some(c) => c.to_string(),
        None => "—".to_string(), // killed runs have no meaningful code
    };

    format!(
        "# Run — {started}\n\n\
         - **command:** `{command}`\n\
         - **interpreter:** {interp}\n\
         - **started:** {started}\n\
         - **finished:** {finished}\n\
         - **duration:** {duration_ms} ms\n\
         - **exit code:** {code}\n\
         - **status:** {status}\n\n\
         ## Output\n\n\
         ```\n{output}\n```\n",
        started = r.started,
        command = r.command,
        interp = r.interp,
        finished = r.finished,
        duration_ms = r.duration_ms,
        code = code,
        status = r.status,
        output = r.output,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_a_report_we_can_read_back() {
        let dir = std::env::temp_dir().join("easishell_history_test");
        let report = RunReport {
            command: "Write-Output hi",
            interp: "powershell",
            started: "2026-05-31 10:00:00",
            finished: "2026-05-31 10:00:01",
            duration_ms: 1234,
            exit_code: Some(0),
            status: "completed",
            output: "hi\r\n",
        };

        let path = write_report(&dir, &report).expect("should write the log");
        assert!(path.exists(), "log file wasnt created");

        let body = std::fs::read_to_string(&path).unwrap();
        assert!(body.contains("Write-Output hi"), "command missing from log");
        assert!(body.contains("hi"), "output missing from log");
        assert!(body.contains("completed"), "status missing from log");

        std::fs::remove_file(&path).ok(); // tidy up after ourselves
    }
}


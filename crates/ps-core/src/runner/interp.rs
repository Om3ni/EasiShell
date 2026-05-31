//! Locating the PowerShell interpreter.
//!
//! Prefer PowerShell 7 (`pwsh`) when present — it emits ANSI color over a pipe,
//! which Windows PowerShell 5.1 does not. Otherwise fall back to the 5.1
//! `powershell.exe` that always ships with Windows.

use std::path::PathBuf;

/// Which PowerShell flavor was resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterpKind {
    /// PowerShell 7+ (`pwsh`).
    Pwsh,
    /// Windows PowerShell 5.1 (`powershell.exe`).
    WindowsPowerShell,
}

/// A resolved interpreter: its executable path and which flavor it is.
#[derive(Debug, Clone)]
pub struct Interpreter {
    pub path: PathBuf,
    pub kind: InterpKind,
}

impl Interpreter {
    /// A short label for display/logging.
    pub fn label(&self) -> &'static str {
        match self.kind {
            InterpKind::Pwsh => "pwsh",
            InterpKind::WindowsPowerShell => "powershell",
        }
    }
}

/// Resolve the interpreter to use, preferring a *real* `pwsh`.
#[cfg(windows)]
pub fn detect() -> Interpreter {
    if let Some(path) = find_pwsh() {
        return Interpreter {
            path,
            kind: InterpKind::Pwsh,
        };
    }
    let fallback =
        PathBuf::from(r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe");
    let path = which::which("powershell").unwrap_or(fallback);
    Interpreter {
        path,
        kind: InterpKind::WindowsPowerShell,
    }
}

/// Find a genuine `pwsh.exe`, skipping the Windows App Execution Alias stub.
///
/// The alias at `%LOCALAPPDATA%\Microsoft\WindowsApps\pwsh.exe` is a 0-byte
/// reparse point that works in an interactive shell but exits silently when
/// launched with redirected stdio (our case), so it must be rejected.
#[cfg(windows)]
fn find_pwsh() -> Option<PathBuf> {
    // 1. Standard MSI install locations (these are real executables).
    for var in ["ProgramFiles", "ProgramW6432", "ProgramFiles(x86)"] {
        if let Some(base) = std::env::var_os(var) {
            let cand = PathBuf::from(base).join("PowerShell").join("7").join("pwsh.exe");
            if is_real_exe(&cand) {
                return Some(cand);
            }
        }
    }
    // 2. PATH lookup, rejecting the WindowsApps alias / any 0-byte stub.
    if let Ok(p) = which::which("pwsh") {
        if is_real_exe(&p) && !is_windows_apps_alias(&p) {
            return Some(p);
        }
    }
    None
}

#[cfg(windows)]
fn is_real_exe(p: &std::path::Path) -> bool {
    std::fs::metadata(p)
        .map(|m| m.is_file() && m.len() > 0)
        .unwrap_or(false)
}

#[cfg(windows)]
fn is_windows_apps_alias(p: &std::path::Path) -> bool {
    p.to_string_lossy().to_lowercase().contains("windowsapps")
}

/// Non-Windows fallback (dev/test on other platforms): just look for `pwsh`.
#[cfg(not(windows))]
pub fn detect() -> Interpreter {
    let path = std::env::var_os("PWSH_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("pwsh"));
    Interpreter {
        path,
        kind: InterpKind::Pwsh,
    }
}

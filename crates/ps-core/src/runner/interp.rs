//! Working out which PowerShell to actually run.
//!
//! We want pwsh 7 if its around, mainly because it emits ANSI color over a pipe and
//! Windows PowerShell 5.1 just doesnt — so under 5.1 the console ends up colorless.
//! If theres no real pwsh we fall back to the 5.1 powershell.exe, which ships with
//! every Windows box so it's always there.

use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterpKind {
    /// PowerShell 7+ (`pwsh`).
    Pwsh,
    /// Windows PowerShell 5.1 (`powershell.exe`).
    WindowsPowerShell,
}

/// A resolved interpreter — where the exe is and which flavor we landed on.
#[derive(Debug, Clone)]
pub struct Interpreter {
    pub path: PathBuf,
    pub kind: InterpKind,
}

impl Interpreter {
    pub fn label(&self) -> &'static str {
        match self.kind {
            InterpKind::Pwsh => "pwsh",
            InterpKind::WindowsPowerShell => "powershell",
        }
    }
}

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

/// Find a genuine pwsh.exe and dodge the App Execution Alias.
///
/// This one's a real gotcha. when PowerShell 7 comes from the Store, the only thing
/// on PATH is `%LOCALAPPDATA%\Microsoft\WindowsApps\pwsh.exe` — and thats a 0-byte
/// reparse stub. it works fine if you type `pwsh` in a terminal, but launch it with
/// redirected stdio like we do and it just exits silently with no output. so we go
/// looking for the actual install first, and if PATH only hands back the stub we
/// reject it (0 bytes, or sitting under WindowsApps) and fall through to 5.1.
#[cfg(windows)]
fn find_pwsh() -> Option<PathBuf> {
    // the normal MSI install spots first — these are honest, real exes.
    for var in ["ProgramFiles", "ProgramW6432", "ProgramFiles(x86)"] {
        if let Some(base) = std::env::var_os(var) {
            let cand = PathBuf::from(base).join("PowerShell").join("7").join("pwsh.exe");
            if is_real_exe(&cand) {
                return Some(cand);
            }
        }
    }
    // otherwise try PATH, but throw out the alias stub if thats all we get.
    if let Ok(p) = which::which("pwsh") {
        if is_real_exe(&p) && !is_windows_apps_alias(&p) {
            return Some(p);
        }
    }
    None
}

// the alias is 0 bytes, so the length check alone already catches it — the path check
// below is just belt-and-suspenders in case the stub ever stops being empty.
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

// only here so the crate still builds/tests on non-Windows dev boxes. real detection
// is the Windows path above.
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

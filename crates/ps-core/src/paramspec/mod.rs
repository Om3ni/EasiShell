//! What inputs a runnable declares.
//!
//! One shared model so the same form renders a user script's params and a built-in tool's
//! params, and so the `.ps1` import (Phase 5) can parse a `param()` block straight into
//! these. At runtime the *values* a user fills in get turned into `-Name value` argv
//! elements — but thats the app's job; this is just the shape.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../src/lib/ipc/bindings/")]
#[serde(rename_all = "lowercase")]
pub enum ParamKind {
    Text,
    Number,
    Dropdown,
    Checkbox,
    Path,
    Multiline,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../src/lib/ipc/bindings/")]
#[serde(rename_all = "camelCase")]
pub struct ParamSpec {
    /// the PowerShell parameter name, no leading dash (we add the `-` when we build argv).
    pub name: String,
    /// what the form puts next to the input.
    pub label: String,
    pub kind: ParamKind,
    /// starting value, as text. for a checkbox it's "true"/"false". empty means no default.
    #[serde(default)]
    pub default: String,
    /// choices for a dropdown; ignored for every other kind.
    #[serde(default)]
    pub options: Vec<String>,
    #[serde(default)]
    pub required: bool,
}

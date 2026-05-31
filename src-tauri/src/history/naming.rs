//! Naming the per-run log file.
//!
//! Format the user asked for: `YYYY.MM.DD.HHMM.md` in the machine's *local* time on a
//! 24-hour clock. Two runs inside the same minute would otherwise clobber each other,
//! so we take a "does this name exist already" check and tack on `-2`, `-3`… until we
//! find a free one.

use chrono::Local;

pub fn log_filename(exists: impl Fn(&str) -> bool) -> String {
    let stamp = Local::now().format("%Y.%m.%d.%H%M").to_string();

    let first = format!("{stamp}.md");
    if !exists(&first) {
        return first;
    }

    // same-minute collision — start suffixing. theres no realistic ceiling here so
    // just keep counting; in practice you'll never get past a handful.
    let mut n = 2;
    loop {
        let candidate = format!("{stamp}-{n}.md");
        if !exists(&candidate) {
            return candidate;
        }
        n += 1;
    }
}

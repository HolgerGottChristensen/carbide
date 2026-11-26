//! Opt-in timing, for finding where frames go. Off unless the environment variable
//! `CARBIDE_3D_PROFILE` is set when the program starts:
//! - `CARBIDE_3D_PROFILE=1`: CPU timings, waits for the GPU, and scene sizes.
//! - `CARBIDE_3D_PROFILE=gpu`: as `1`, and each view also waits for the GPU after submitting its
//!   frame, timing the GPU's work for it. Waiting slows rendering down: use it to measure only.
//!
//! Once a second, it prints a summary to stderr: each measurement's count, average, maximum and
//! total per second, grouped by view. A view's `render` count is its frames per second.

use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

/// What is measured.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Mode {
    Off,
    Cpu,
    /// CPU timings, plus a wait for the GPU after each view.
    Gpu,
}

/// The mode, from `CARBIDE_3D_PROFILE`, read once.
pub fn mode() -> Mode {
    static MODE: OnceLock<Mode> = OnceLock::new();
    *MODE.get_or_init(|| match std::env::var("CARBIDE_3D_PROFILE").as_deref() {
        Ok("gpu") | Ok("GPU") => Mode::Gpu,
        Ok("") | Ok("0") | Err(_) => Mode::Off,
        Ok(_) => Mode::Cpu,
    })
}

/// Whether anything is measured.
pub fn enabled() -> bool {
    mode() != Mode::Off
}

#[derive(Debug, Default)]
struct Stat {
    count: u64,
    total: f64,
    max: f64,
    /// A time in milliseconds, or a plain number.
    time: bool,
}

#[derive(Debug)]
struct Profile {
    stats: BTreeMap<String, Stat>,
    since: Instant,
    /// Put in front of what is recorded: the view rendering now.
    scope: String,
}

fn profile() -> &'static Mutex<Profile> {
    static PROFILE: OnceLock<Mutex<Profile>> = OnceLock::new();
    PROFILE.get_or_init(|| Mutex::new(Profile { stats: BTreeMap::new(), since: Instant::now(), scope: String::new() }))
}

fn add(name: &str, value: f64, time: bool) {
    let mut profile = profile().lock().unwrap();
    let key = if profile.scope.is_empty() { name.to_string() } else { format!("{}: {}", profile.scope, name) };
    let stat = profile.stats.entry(key).or_default();
    stat.count += 1;
    stat.total += value;
    stat.max = stat.max.max(value);
    stat.time = time;
}

/// Record how long something took.
pub fn record(name: &str, duration: Duration) {
    if enabled() {
        add(name, duration.as_secs_f64() * 1000.0, true);
    }
}

/// Record a number, such as how many meshes were drawn.
pub fn record_value(name: &str, value: f64) {
    if enabled() {
        add(name, value, false);
    }
}

/// Run `f`, recording how long it took.
pub fn time<R>(name: &str, f: impl FnOnce() -> R) -> R {
    if !enabled() {
        return f();
    }
    let start = Instant::now();
    let result = f();
    record(name, start.elapsed());
    result
}

/// Record what follows under `scope`, such as a view, until the next call.
pub fn set_scope(scope: impl Into<String>) {
    if enabled() {
        profile().lock().unwrap().scope = scope.into();
    }
}

/// Print a line once, such as which GPU is used.
pub fn note_once(key: &'static str, line: impl FnOnce() -> String) {
    static NOTED: OnceLock<Mutex<Vec<&'static str>>> = OnceLock::new();
    if !enabled() {
        return;
    }
    let mut noted = NOTED.get_or_init(|| Mutex::new(vec![])).lock().unwrap();
    if !noted.contains(&key) {
        noted.push(key);
        eprintln!("carbide_3d profile: {}", line());
    }
}

/// Print the summary if a second has passed since the last one, and start a new one.
pub fn report_if_due() {
    if !enabled() {
        return;
    }
    let mut profile = profile().lock().unwrap();
    let elapsed = profile.since.elapsed().as_secs_f64();
    if elapsed < 1.0 {
        return;
    }

    let mut out = format!("carbide_3d profile, the last {elapsed:.2} s ({:?} mode):\n", mode());
    out.push_str(&format!("  {:<52} {:>7} {:>10} {:>10} {:>10}\n", "", "count", "average", "max", "per second"));
    for (name, stat) in &profile.stats {
        let average = stat.total / stat.count.max(1) as f64;
        if stat.time {
            out.push_str(&format!("  {:<52} {:>7} {:>8.2}ms {:>8.2}ms {:>8.1}ms\n", name, stat.count, average, stat.max, stat.total / elapsed));
        } else {
            out.push_str(&format!("  {:<52} {:>7} {:>10.1} {:>10.1}\n", name, stat.count, average, stat.max));
        }
    }
    eprint!("{out}");

    profile.stats.clear();
    profile.since = Instant::now();
}

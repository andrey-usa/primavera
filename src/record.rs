//! `primavera record <url>`: a browser to use by hand; what you do in it
//! becomes a scenario. A script in every page of the browser notes each
//! click, field change, check, select, upload and Enter as it happens,
//! with the locators that find the element (navigera's `locate`, run in
//! the page at that moment, so a click that navigates away is still
//! known), and says so on the console; primavera reads it from there and
//! writes the scenario the way `from-session` does. The file is rewritten
//! after every step, so stopping with Ctrl-C keeps what was done.

use std::time::{Duration, Instant};

use navigera::protocol::{Driver, SessionConfig};
use serde_json::{json, Value};

/// What the page says before each note on the console.
const MARK: &str = "__pv_rec ";

/// The script every document gets: the notes, `MARK` + JSON.
fn recorder() -> String {
    format!(
        r#"(() => {{
  if (window.__pvRecorder) return;
  window.__pvRecorder = true;
  const locate = {locate};
  let seq = 0;
  const note = (el, step) => {{
    const n = ++seq, at = Date.now(), url = location.href;
    locate(el).then((r) => console.debug({mark} + JSON.stringify({{ ...step, n, at, url, locators: r.locators || [] }})),
      () => console.debug({mark} + JSON.stringify({{ ...step, n, at, url, locators: [] }})));
  }};
  const target = (e) => {{ let el = e.composedPath ? e.composedPath()[0] : e.target; return el && el.nodeType === 1 ? el : el && el.parentElement; }};
  const FIELD = 'input:not([type=button]):not([type=submit]):not([type=reset]):not([type=image]),textarea,select,option,[contenteditable]';
  const TOGGLE = 'input[type=checkbox],input[type=radio]';
  document.addEventListener('click', (e) => {{
    const el = target(e);
    if (!el) return;
    const act = el.closest('a,button,input,summary,label,[role],[onclick],[tabindex]') || el;
    // Into a field (typing follows) or a toggle (its change is the step):
    // not a click of its own.
    if (act.matches(FIELD) || act.matches(TOGGLE)) return;
    if (act.tagName === 'LABEL' && act.control) return;
    note(act, {{ op: 'click' }});
  }}, true);
  document.addEventListener('change', (e) => {{
    const el = target(e);
    if (!el) return;
    if (el.matches(TOGGLE)) note(el, {{ op: 'set-checked', checked: el.checked }});
    else if (el.tagName === 'SELECT') note(el, {{ op: 'select', value: [...el.selectedOptions].map((o) => (o.label || o.text).trim()) }});
    else if (el.type === 'file') note(el, {{ op: 'upload', files: [...el.files].map((f) => f.name) }});
    else if ('value' in el) note(el, {{ op: 'fill', value: el.type === 'password' ? '${{secret:PASSWORD}}' : el.value }});
  }}, true);
  document.addEventListener('keydown', (e) => {{
    const el = target(e);
    if (e.key !== 'Enter' || !el || !el.matches('input')) return;
    note(el, {{ op: 'fill', value: el.type === 'password' ? '${{secret:PASSWORD}}' : el.value }});
    note(el, {{ op: 'press', key: 'Enter' }});
  }}, true);
}})()"#,
        locate = navigera::cdp::locate_js(),
        mark = serde_json::to_string(MARK).unwrap_or_default(),
    )
}

/// The notes as history entries (`from-session`'s input): in the order
/// they happened, a field filled twice in a row once, each entry's `url`
/// where the page was after it (the next note's, or `now` for the last).
pub fn history(start: &str, mut notes: Vec<Value>, now: &str) -> Vec<Value> {
    notes.sort_by_key(|n| (n["at"].as_u64().unwrap_or(0), n["n"].as_u64().unwrap_or(0)));
    let same = |a: &Value, b: &Value| a["op"] == "fill" && b["op"] == "fill" && a["locators"] == b["locators"];
    let mut kept: Vec<Value> = Vec::new();
    for n in notes {
        match kept.last_mut() {
            Some(last) if same(last, &n) => *last = n,
            _ => kept.push(n),
        }
    }
    let mut out = vec![json!({"command": {"op": "goto", "url": start}, "url": start})];
    for (i, n) in kept.iter().enumerate() {
        let after = kept.get(i + 1).and_then(|m| m["url"].as_str()).unwrap_or(now);
        let mut command = json!({"op": n["op"], "ref": 0});
        for k in ["value", "checked", "files", "key"] {
            if !n[k].is_null() {
                command[k] = n[k].clone();
            }
        }
        out.push(json!({"command": command, "locators": n["locators"], "url": after}));
    }
    out
}

/// `primavera record <url> [--out <file.pv>] [--scenario <name>] [--headless] [--for <s>]`.
pub fn main(args: &[String]) -> Result<String, String> {
    let flag = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1));
    let values: Vec<&String> = [flag("--out"), flag("--scenario"), flag("--for")].into_iter().flatten().collect();
    let url = args
        .iter()
        .find(|a| !a.starts_with("--") && !values.contains(a))
        .ok_or("usage: primavera record <url> [--out <file.pv>] [--scenario <name>] [--headless] [--for <seconds>]")?;
    let url = navigera::protocol::normalize_url(url);
    let out = flag("--out");
    let scenario = flag("--scenario").cloned().unwrap_or_else(|| "Recorded".to_string());
    let until = match flag("--for") {
        Some(s) => Some(Duration::from_secs_f64(s.parse().map_err(|_| format!("--for takes seconds, not {s:?}"))?)),
        None => None,
    };
    let headless = args.iter().any(|a| a == "--headless");
    let config = SessionConfig { headed: !headless, ..SessionConfig::default() };
    let mut driver = Driver::launch(&config).map_err(|e| format!("the browser didn't start: {e:#}"))?;
    let run = |driver: &mut Driver, words: &[&str]| -> Result<Value, String> {
        let words: Vec<String> = words.iter().map(|w| w.to_string()).collect();
        let cmd = crate::runner::parse(&words)?;
        driver.run(&cmd).map_err(|e| format!("{e:#}"))
    };
    run(&mut driver, &["init-script", &recorder()])?;
    run(&mut driver, &["goto", &url])?;
    eprintln!(
        "recording {url}: use the page; {}",
        match (until, out) {
            (Some(d), _) => format!("stops in {} s", d.as_secs_f64()),
            (None, Some(o)) => format!("close the browser or press Ctrl-C to stop ({o} is kept up to date)"),
            (None, None) => "close the browser or press Ctrl-C to stop".into(),
        }
    );
    let started = Instant::now();
    let mut notes: Vec<Value> = Vec::new();
    let mut here = url.clone();
    let mut text = crate::from_session::render(&history(&url, Vec::new(), &url), "recorded", &scenario);
    loop {
        std::thread::sleep(Duration::from_millis(200));
        let Ok(console) = run(&mut driver, &["console", "--json", "--clear"]) else { break };
        let before = notes.len();
        for m in console.as_array().into_iter().flatten() {
            if let Some(json) = m["text"].as_str().and_then(|t| t.strip_prefix(MARK)) {
                if let Ok(n) = serde_json::from_str::<Value>(json) {
                    eprintln!("  {} {}", n["op"].as_str().unwrap_or(""), n["locators"][0]["flags"]);
                    notes.push(n);
                }
            }
        }
        if let Ok(u) = run(&mut driver, &["url"]) {
            here = u["url"].as_str().unwrap_or(&here).to_string();
        }
        if notes.len() != before || text.is_empty() {
            text = crate::from_session::render(&history(&url, notes.clone(), &here), "recorded", &scenario);
            if let Some(path) = out {
                std::fs::write(path, &text).map_err(|e| format!("{path}: {e}"))?;
            }
        }
        if until.is_some_and(|d| started.elapsed() >= d) {
            break;
        }
    }
    text = crate::from_session::render(&history(&url, notes, &here), "recorded", &scenario);
    driver.session().close();
    match out {
        Some(path) => {
            std::fs::write(path, &text).map_err(|e| format!("{path}: {e}"))?;
            Ok(format!("wrote {path}; check it with `primavera lint {path}`, then run it\n"))
        }
        None => Ok(text),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notes_become_steps_in_order_with_where_each_left_the_page() {
        let loc = |n: &str| json!([{"kind": "role", "flags": ["--role", "textbox", "--name", n]}]);
        let notes = vec![
            json!({"op": "click", "n": 3, "at": 30, "url": "http://x/login", "locators": [{"kind": "role", "flags": ["--role", "button", "--name", "Sign in"]}]}),
            json!({"op": "fill", "value": "a", "n": 1, "at": 10, "url": "http://x/login", "locators": loc("Email")}),
            json!({"op": "fill", "value": "ada", "n": 2, "at": 20, "url": "http://x/login", "locators": loc("Email")}),
            json!({"op": "set-checked", "checked": true, "n": 4, "at": 40, "url": "http://x/home", "locators": [{"kind": "label", "flags": ["--label", "Remember"]}]}),
        ];
        let h = history("http://x/login", notes, "http://x/home");
        let pv = crate::from_session::render(&h, "recorded", "Signs in");
        assert!(
            pv.ends_with(
                "scenario \"Signs in\"\n  goto /login\n  fill --role textbox --name Email ada\n  \
                 click --role button --name \"Sign in\"\n  expect url ends-with /home\n  check --label Remember\n"
            ),
            "{pv}"
        );
        assert!(recorder().contains("async function (el)"), "the page gets navigera's locate");
    }
}

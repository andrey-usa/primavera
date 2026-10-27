//! A project can bring its own `.primavera` (a cloned repository): primavera
//! doesn't write evidence, timings or sign-in state through a link there.

#![cfg(unix)]

use std::process::Command;

/// A fresh directory under the system's temporary one, removed on drop.
struct Dir(std::path::PathBuf);

impl Dir {
    fn new(name: &str) -> Dir {
        let path = std::env::temp_dir().join(format!("pv-state-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        Dir(path)
    }
    fn path(&self) -> &std::path::Path {
        &self.0
    }
}

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn primavera(dir: &std::path::Path, args: &[&str]) -> (Option<i32>, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_primavera")).args(args).current_dir(dir).output().expect("run");
    (out.status.code(), String::from_utf8_lossy(&out.stderr).into_owned())
}

#[test]
fn a_linked_state_directory_is_refused() {
    let project = Dir::new("project");
    let elsewhere = Dir::new("elsewhere");
    std::fs::write(project.path().join("t.pv"), "scenario \"x\"\n  goto /\n").unwrap();
    std::os::unix::fs::symlink(elsewhere.path(), project.path().join(".primavera")).unwrap();
    for args in [&["run", "t.pv"][..], &["fix", "t.pv:2"], &["lint", "t.pv"]] {
        let (code, err) = primavera(project.path(), args);
        assert_eq!(code, Some(2), "{args:?}: {err}");
        assert!(err.contains(".primavera is a link"), "{args:?}: {err}");
    }
    assert_eq!(std::fs::read_dir(elsewhere.path()).unwrap().count(), 0, "something landed through the link");
    // Help and the skill don't touch it.
    assert_eq!(primavera(project.path(), &["--help"]).0, Some(0));

    // A real `.primavera` with `runs` linked.
    std::fs::remove_file(project.path().join(".primavera")).unwrap();
    std::fs::create_dir(project.path().join(".primavera")).unwrap();
    std::os::unix::fs::symlink(elsewhere.path(), project.path().join(".primavera/runs")).unwrap();
    let (code, err) = primavera(project.path(), &["run", "t.pv"]);
    assert_eq!(code, Some(2), "{err}");
    assert!(err.contains(".primavera/runs is a link"), "{err}");
    assert_eq!(std::fs::read_dir(elsewhere.path()).unwrap().count(), 0);
}

// The binary prints no output in the Linux ARM tests run under cross emulation.
#![cfg(not(all(target_os = "linux", any(target_arch = "arm", target_arch = "aarch64"))))]

use std::io::Write;
use std::process::{Command, Stdio};

const PATCH: &str = "\
diff --git a/file.txt b/file.txt
--- a/file.txt
+++ b/file.txt
@@ -1 +1 @@
-old
+new
";

fn render(advertised: Option<&str>) -> String {
    let mut command = Command::new(env!("CARGO_BIN_EXE_delta"));
    command
        .args(["--no-gitconfig", "--dark", "--paging=never", "--color-only"])
        .env("DELTA_FEATURES", "")
        .env_remove("OSC1717")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(advertised) = advertised {
        command.env("OSC1717", advertised);
    }
    let mut child = command.spawn().unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(PATCH.as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn metadata_requires_a_supported_host() {
    for advertised in [None, Some("V2"), Some("invalid")] {
        assert!(!render(advertised).contains("\x1b]1717;"));
    }
}

#[test]
fn supported_host_receives_handshake_and_patch_identities() {
    for advertised in ["V1", "V2,V1"] {
        let output = render(Some(advertised));
        assert!(output.starts_with("\x1b]1717;1\x1b\\"));
        assert!(output.contains("\x1b]1717;1;d;1;1;file.txt\x1b\\"));
        assert!(output.contains("\x1b]1717;1;a;1;;file.txt\x1b\\"));
    }
}

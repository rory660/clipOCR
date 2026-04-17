use assert_cmd::Command;

#[test]
fn version_flag() {
    Command::cargo_bin("clipocr")
        .unwrap()
        .arg("--version")
        .assert()
        .success();
}

#[test]
fn help_flag() {
    Command::cargo_bin("clipocr")
        .unwrap()
        .arg("--help")
        .assert()
        .success();
}

#[cfg(target_os = "macos")]
#[test]
fn ocr_hello_fixture_via_example() {
    // The example binary bypasses the clipboard and runs the Vision engine
    // directly on the fixture, so this test is deterministic in CI.
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hello.png");
    if !fixture.exists() {
        eprintln!("skipping: fixture missing");
        return;
    }
    let out = Command::new(env!("CARGO"))
        .args([
            "run",
            "--quiet",
            "--example",
            "ocr_file",
            "--",
            fixture.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Hello, world."), "got: {stdout}");
}

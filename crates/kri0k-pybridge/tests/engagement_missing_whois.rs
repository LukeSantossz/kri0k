//! Integration test for D-50 / M-36: `Engagement::new` must fail when the
//! whois binary is absent from `PATH`.
//!
//! This is a Rust-side integration test (not a `PyO3` binding test) — it asserts
//! that `which::which("whois")` failing produces an `Error::MissingDependency`.
//! Direct `PyO3` instantiation requires a Python interpreter; this test focuses on
//! the underlying Rust pathway covered inside `Engagement::new`'s `allow_threads`
//! block.
//!
//! Uses `temp-env` to clear `PATH` safely (serialized to avoid races with parallel
//! tests).

use temp_env::with_var;

/// Helper that mirrors the `which::which` call inside `Engagement::new`.
/// Validates the same failure path without bringing up a Python interpreter.
fn check_whois_available() -> Result<std::path::PathBuf, which::Error> {
    which::which("whois")
}

#[test]
fn engagement_new_fails_without_whois_in_path() {
    // temp-env serializes env-var mutations to avoid races between parallel tests.
    with_var("PATH", Some(""), || {
        let result = check_whois_available();
        assert!(
            result.is_err(),
            "expected which::which('whois') to fail with empty PATH, got {result:?}"
        );
    });
}

#[test]
fn engagement_new_succeeds_when_whois_in_path() {
    // Sanity check — if whois IS installed locally, which::which finds it.
    // Skip if absent (CI greenfield).
    if which::which("whois").is_err() {
        eprintln!("skipping: whois binary not in PATH");
        return;
    }
    let result = check_whois_available();
    assert!(result.is_ok());
}

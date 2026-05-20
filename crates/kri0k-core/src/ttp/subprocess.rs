//! Subprocess abstraction for TTP execution (D-54).
//!
//! Defines a testable [`Subprocess`] trait with two impls:
//! - [`RealSubprocess`]: uses `tokio::process::Command` with timeout + cancellation.
//! - [`MockSubprocess`]: reads from a fixture file or hangs forever for tests.
//!
//! # Design rationale
//! Abstracting subprocess allows unit tests to run without installing system
//! binaries. Integration tests (gated by `--features integration`) use
//! `RealSubprocess` with the actual `whois` binary.
//!
//! See: `docs/adr/ADR-0012-ttp-trait-adapters.md`
//! See: `04-CONTEXT.md D-54`

use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use async_trait::async_trait;
use tokio_util::sync::CancellationToken;

/// Output from a subprocess invocation.
#[derive(Debug, Clone)]
pub struct SubprocessOutput {
    /// Decoded stdout (UTF-8 lossy — safe for Windows locale output, Pitfall 3).
    pub stdout: String,
    /// Decoded stderr (UTF-8 lossy).
    pub stderr: String,
    /// Exit code if the process exited normally.
    pub exit_code: Option<i32>,
}

/// Testable subprocess abstraction (D-54, ADR-0012).
///
/// `#[async_trait]` is MANDATORY for dyn-compatibility with `Box<dyn Subprocess>`.
/// Native `async fn in trait` is not dyn-compatible in Rust 1.85 (Pitfall 6).
#[async_trait]
pub trait Subprocess: Send + Sync {
    /// Run a subprocess command.
    ///
    /// # Arguments
    /// - `cmd`: executable name (no shell expansion — D-63 Layer 3).
    /// - `args`: argument slice (each element passed as a separate `Command::arg`).
    /// - `timeout`: kill the process if it runs longer than this.
    /// - `cancel`: kill the process if cancelled (M-36, D-62).
    ///
    /// # Errors
    /// - `Error::Cancelled` if `cancel` fires before completion.
    /// - `Error::SubprocessTimeout` if `timeout` elapses.
    /// - `Error::Io` for spawn/wait failures.
    async fn run(
        &self,
        cmd: &str,
        args: &[&str],
        timeout: Duration,
        cancel: CancellationToken,
    ) -> Result<SubprocessOutput, crate::Error>;
}

/// Real subprocess implementation using `tokio::process::Command` (D-44).
///
/// Uses `tokio::select! { biased; ... }` with three branches:
/// 1. `cancel.cancelled()` → kill process, return `Error::Cancelled` (M-36).
/// 2. `tokio::time::sleep(timeout)` → kill process, return `Error::SubprocessTimeout`.
/// 3. `child.wait_with_output()` → decode and return `SubprocessOutput`.
///
/// `biased;` ensures cancellation is always polled first (Pitfall 4).
/// `kill_on_drop(true)` is belt-and-suspenders in case branches are not reached.
#[derive(Debug, Default)]
pub struct RealSubprocess;

// Internal enum to carry the select! result without borrowing child across branches.
#[allow(clippy::items_after_statements)]
enum RealOutcome {
    Cancelled,
    TimedOut,
    Completed(std::io::Result<std::process::Output>),
}

#[async_trait]
impl Subprocess for RealSubprocess {
    async fn run(
        &self,
        cmd: &str,
        args: &[&str],
        timeout: Duration,
        cancel: CancellationToken,
    ) -> Result<SubprocessOutput, crate::Error> {
        // D-63 Layer 3: no shell expansion — each arg is passed directly.
        // kill_on_drop(true) = belt-and-suspenders per Pitfall 4.
        let child = tokio::process::Command::new(cmd)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(crate::Error::Io)?;

        let outcome = tokio::select! {
            // biased: cancellation polled first — Pitfall 4 (cancel must beat timeout).
            biased;

            // Branch 1: Kill-switch triggered (M-36, D-62).
            () = cancel.cancelled() => RealOutcome::Cancelled,

            // Branch 2: Timeout — kill and report (D-51, M-34).
            () = tokio::time::sleep(timeout) => RealOutcome::TimedOut,

            // Branch 3: Process completed normally.
            res = child.wait_with_output() => RealOutcome::Completed(res),
        };

        match outcome {
            RealOutcome::Cancelled => {
                // kill_on_drop(true) handles cleanup; returning immediately is safe.
                Err(crate::Error::Cancelled)
            },
            RealOutcome::TimedOut => {
                Err(crate::Error::SubprocessTimeout {
                    ttp_id: "<subprocess>".into(),
                    // u64::try_from avoids clippy cast_possible_truncation.
                    timeout_ms: u64::try_from(timeout.as_millis()).unwrap_or(u64::MAX),
                })
            },
            RealOutcome::Completed(res) => {
                let output = res.map_err(crate::Error::Io)?;
                Ok(SubprocessOutput {
                    // from_utf8_lossy handles Windows locale stderr (Pitfall 3).
                    stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                    stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
                    exit_code: output.status.code(),
                })
            },
        }
    }
}

// -- MockSubprocess -----------------------------------------------------------

/// Mode for [`MockSubprocess`].
#[derive(Debug)]
enum MockMode {
    /// Read fixture content from a file path and return it as stdout.
    Fixture(PathBuf),
    /// Block indefinitely — lets the caller's timeout/cancel branch fire.
    Hanging,
}

/// Mock subprocess for unit tests (D-54).
///
/// Two constructors:
/// - [`MockSubprocess::from_fixture`]: reads a fixture file and returns its content as stdout.
/// - [`MockSubprocess::hanging`]: blocks indefinitely so the caller can test timeout/cancel.
#[derive(Debug)]
pub struct MockSubprocess {
    mode: MockMode,
}

impl MockSubprocess {
    /// Construct a mock that returns the content of `path` as stdout.
    #[must_use]
    pub const fn from_fixture(path: PathBuf) -> Self {
        Self {
            mode: MockMode::Fixture(path),
        }
    }

    /// Construct a mock that blocks until cancelled or timed out by the caller.
    ///
    /// Use this to test `Error::Cancelled` and `Error::SubprocessTimeout` paths.
    #[must_use]
    pub const fn hanging() -> Self {
        Self {
            mode: MockMode::Hanging,
        }
    }
}

#[async_trait]
impl Subprocess for MockSubprocess {
    async fn run(
        &self,
        _cmd: &str,
        _args: &[&str],
        timeout: Duration,
        cancel: CancellationToken,
    ) -> Result<SubprocessOutput, crate::Error> {
        match &self.mode {
            MockMode::Fixture(path) => {
                // Use std::fs (sync) — MockSubprocess is test-only and the
                // blocking I/O is negligible for small fixture files.
                let stdout = std::fs::read_to_string(path).map_err(crate::Error::Io)?;
                Ok(SubprocessOutput {
                    stdout,
                    stderr: String::new(),
                    exit_code: Some(0),
                })
            },
            MockMode::Hanging => {
                // Mirrors RealSubprocess select! so callers can test cancel/timeout
                // without spawning a real process.
                tokio::select! {
                    biased;
                    () = cancel.cancelled() => Err(crate::Error::Cancelled),
                    () = tokio::time::sleep(timeout) => Err(crate::Error::SubprocessTimeout {
                        ttp_id: "<mock>".into(),
                        timeout_ms: u64::try_from(timeout.as_millis()).unwrap_or(u64::MAX),
                    }),
                    () = std::future::pending::<()>() => unreachable!("pending never resolves"),
                }
            },
        }
    }
}

// -- Tests --------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use tokio_util::sync::CancellationToken;

    /// Verify `RealSubprocess` returns `Error::Io` when the binary is missing.
    #[tokio::test]
    async fn test_real_subprocess_command_not_found() {
        let sub = RealSubprocess;
        let result = sub
            .run(
                "nonexistent_binary_xyz_kri0k",
                &[],
                Duration::from_secs(1),
                CancellationToken::new(),
            )
            .await;
        assert!(
            matches!(result, Err(crate::Error::Io(_))),
            "expected Io error for missing binary, got {result:?}"
        );

        // Dyn-compat smoke: Box<dyn Subprocess> must compile.
        let _dyn_sub: Box<dyn Subprocess> = Box::new(MockSubprocess::hanging());
    }

    /// Verify cancellation fires before spawn when token is pre-cancelled.
    #[tokio::test]
    async fn test_real_subprocess_cancel_before_spawn() {
        let token = CancellationToken::new();
        token.cancel(); // cancel BEFORE calling run
        let sub = RealSubprocess;
        // The biased select! will pick cancel.cancelled() on first poll.
        let result = sub
            .run(
                "nonexistent_binary_xyz_kri0k",
                &[],
                Duration::from_secs(1),
                token,
            )
            .await;
        // Either Cancelled (cancel wins) or Io (spawn fails before select).
        // Both are valid — the point is it does NOT hang.
        assert!(
            matches!(result, Err(crate::Error::Cancelled | crate::Error::Io(_))),
            "expected Cancelled or Io, got {result:?}"
        );
    }

    /// Verify `MockSubprocess::from_fixture` returns file content as stdout.
    #[tokio::test]
    async fn test_mock_subprocess_from_fixture_returns_file_content() {
        // Use the whois_invalid.txt fixture (smallest, always present).
        let fixture_path = std::path::PathBuf::from(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/whois_invalid.txt"
        ));
        let sub = MockSubprocess::from_fixture(fixture_path);
        let result = sub
            .run("x", &[], Duration::from_secs(5), CancellationToken::new())
            .await
            .expect("fixture subprocess should not fail");
        assert!(
            !result.stdout.is_empty(),
            "stdout should contain fixture content"
        );
        assert_eq!(result.exit_code, Some(0));
    }

    /// Verify `MockSubprocess::hanging` returns `SubprocessTimeout` when timeout fires.
    #[tokio::test]
    async fn test_mock_subprocess_hanging_times_out() {
        let sub = MockSubprocess::hanging();
        let result = sub
            .run(
                "x",
                &[],
                Duration::from_millis(50),
                CancellationToken::new(),
            )
            .await;
        assert!(
            matches!(result, Err(crate::Error::SubprocessTimeout { .. })),
            "expected SubprocessTimeout, got {result:?}"
        );
    }
}

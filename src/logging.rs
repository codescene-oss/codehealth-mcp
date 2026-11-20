use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::EnvFilter;

/// Build a tracing subscriber with stderr output and optional file logging.
///
/// Returns the non-blocking file-appender guard when file logging is
/// active.  The guard must be held for the lifetime of the program so
/// that buffered log entries are flushed on shutdown.
pub(crate) fn build_subscriber(
    log_dir: &Path,
    retention_days: u32,
) -> (
    impl tracing::Subscriber + Send + Sync,
    Option<tracing_appender::non_blocking::WorkerGuard>,
) {
    let env_filter = EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into());
    let stderr_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stderr)
        .with_ansi(false);

    let (file_layer, guard) = prepare_log_dir(log_dir, retention_days)
        .and_then(open_file_appender)
        .map(|appender| {
            let (writer, guard) = tracing_appender::non_blocking(appender);
            let layer = tracing_subscriber::fmt::layer()
                .with_writer(writer)
                .with_ansi(false);
            (layer, guard)
        })
        .unzip();

    let subscriber = tracing_subscriber::registry()
        .with(env_filter)
        .with(stderr_layer)
        .with(file_layer);
    (subscriber, guard)
}

fn open_file_appender(log_dir: PathBuf) -> Option<RollingFileAppender> {
    match RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("mcp.log")
        .build(&log_dir)
    {
        Ok(appender) => Some(appender),
        Err(error) => {
            eprintln!("File logging unavailable in {log_dir:?}; logging to stderr only: {error}");
            None
        }
    }
}

/// Create the log directory and prune old logs when file logging is enabled.
///
/// Returns `None` when file logging is disabled (`retention_days == 0`) or the
/// directory cannot be created.
fn prepare_log_dir(log_dir: &Path, retention_days: u32) -> Option<PathBuf> {
    if retention_days == 0 {
        return None;
    }
    std::fs::create_dir_all(log_dir).ok()?;
    cleanup_old_logs(log_dir, retention_days);
    Some(log_dir.to_path_buf())
}

/// Remove log files older than `retention_days` from the given directory.
fn cleanup_old_logs(log_dir: &Path, retention_days: u32) {
    let cutoff = SystemTime::now() - Duration::from_secs(u64::from(retention_days) * 24 * 60 * 60);
    let entries = match std::fs::read_dir(log_dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        let Ok(modified) = metadata.modified() else {
            continue;
        };
        if modified < cutoff {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DAY: Duration = Duration::from_secs(24 * 60 * 60);

    fn write_log(dir: &Path, name: &str, age: Duration) -> PathBuf {
        let path = dir.join(name);
        let file = std::fs::File::create(&path).unwrap();
        file.set_modified(SystemTime::now() - age).unwrap();
        path
    }

    #[test]
    fn cleanup_removes_only_logs_older_than_retention() {
        let dir = tempfile::tempdir().unwrap();
        let old = write_log(dir.path(), "old.log", 10 * DAY);
        let recent = write_log(dir.path(), "recent.log", DAY);

        cleanup_old_logs(dir.path(), 7);

        assert!(!old.exists());
        assert!(recent.exists());
    }

    #[test]
    fn cleanup_ignores_missing_directory() {
        let dir = tempfile::tempdir().unwrap();
        cleanup_old_logs(&dir.path().join("missing"), 7);
    }

    #[test]
    fn prepare_log_dir_disabled_when_retention_is_zero() {
        let dir = tempfile::tempdir().unwrap();
        let log_dir = dir.path().join("logs");

        assert_eq!(prepare_log_dir(&log_dir, 0), None);
        assert!(!log_dir.exists());
    }

    #[test]
    fn prepare_log_dir_creates_directory_and_prunes_old_logs() {
        let dir = tempfile::tempdir().unwrap();
        let log_dir = dir.path().join("logs");
        std::fs::create_dir_all(&log_dir).unwrap();
        let old = write_log(&log_dir, "old.log", 10 * DAY);

        assert_eq!(prepare_log_dir(&log_dir, 7), Some(log_dir.clone()));
        assert!(!old.exists());
    }

    #[test]
    fn prepare_log_dir_returns_none_when_directory_cannot_be_created() {
        let dir = tempfile::tempdir().unwrap();
        let blocker = dir.path().join("file");
        std::fs::write(&blocker, "").unwrap();

        assert_eq!(prepare_log_dir(&blocker.join("logs"), 7), None);
    }

    #[test]
    fn subscriber_writes_to_file_when_logging_is_available() {
        let dir = tempfile::tempdir().unwrap();
        let (subscriber, guard) = build_subscriber(dir.path(), 7);
        let guard = guard.expect("file logging should be active");
        tracing::subscriber::with_default(subscriber, || tracing::info!("test log entry"));
        drop(guard);

        let log = std::fs::read_dir(dir.path()).unwrap().next().unwrap();
        let content = std::fs::read_to_string(log.unwrap().path()).unwrap();
        assert!(content.contains("test log entry"), "{content}");
    }

    #[test]
    fn subscriber_still_builds_when_daily_log_file_cannot_be_opened() {
        let dir = tempfile::tempdir().unwrap();
        let today = time::OffsetDateTime::now_utc().date();
        let tomorrow = today.next_day().unwrap();
        for date in [today, tomorrow] {
            std::fs::create_dir(dir.path().join(format!("mcp.log.{date}"))).unwrap();
        }

        let (_subscriber, guard) = build_subscriber(dir.path(), 7);
        assert!(guard.is_none(), "file logging should be inactive");
    }
}

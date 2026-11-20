//! Verifies that an unavailable log file does not prevent MCP initialization.

use super::*;

fn block_daily_log_files(log_dir: &Path) {
    // A directory at the log file path prevents opening it even as root.
    let today = time::OffsetDateTime::now_utc().date();
    let tomorrow = today.next_day().expect("tomorrow's date");
    for date in [today, tomorrow] {
        std::fs::create_dir_all(log_dir.join(format!("mcp.log.{date}")))
            .expect("block the daily log file path");
    }
}

pub fn test_startup_with_unavailable_log_file() {
    let (command, mut env, repo_dir, _tmp) = setup();
    let config_dir = ".config_blocked_log";
    super::use_isolated_config_dir(&mut env, &repo_dir, config_dir);
    block_daily_log_files(&repo_dir.join(config_dir).join("logs"));
    env.extend([
        ("CS_LOG_RETENTION_DAYS".into(), "7".into()),
        ("CS_DISABLE_TRACKING".into(), "1".into()),
        ("CS_DISABLE_VERSION_CHECK".into(), "1".into()),
    ]);

    let mut client = make_client(&command, &env, &repo_dir);
    assert!(client.start(), "Server should tolerate a blocked log file");
    let response = client
        .initialize()
        .unwrap_or_else(|err| panic!("Initialize failed: {err}\n{}", client.get_stderr()));
    assert!(response["result"]["serverInfo"].is_object(), "{response}");
    let stderr = client.get_stderr();
    assert!(stderr.contains("logging to stderr only"), "{stderr}");
    assert!(stderr.contains("CodeHealth MCP server started"), "{stderr}");
}

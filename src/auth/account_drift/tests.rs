use std::ffi::OsString;

use super::super::{now_epoch_secs, AuthManager, CredentialResolutionError};
use crate::config::{load, snapshot_client_env_vars};
use crate::errors::CliError;
use crate::test_utils::{clear_token, MockCliRunner, TokenGuard};

const SIGNED_OUT_JSON: &str = r#"{"status":"signed_out"}"#;
const NO_SLOT: &str = r#"OAuth not signed in. Run `cs auth login`., {:status "signed_out"}"#;

struct DriftEnv {
    _lock: TokenGuard<'static>,
    dir: tempfile::TempDir,
    config_dir: Option<OsString>,
    onprem_url: Option<OsString>,
}

impl DriftEnv {
    fn new() -> Self {
        let env = Self {
            _lock: clear_token(),
            dir: tempfile::tempdir().unwrap(),
            config_dir: std::env::var_os("CS_CONFIG_DIR"),
            onprem_url: std::env::var_os("CS_ONPREM_URL"),
        };
        std::env::set_var("CS_CONFIG_DIR", env.dir.path());
        std::env::remove_var("CS_ONPREM_URL");
        std::env::set_var("CS_ACCOUNT_ID", "11");
        std::env::set_var("CS_OAUTH_ACCOUNT_ID", "3834");
        std::env::set_var("CS_OAUTH_TOKEN", "cached-token");
        std::env::set_var("CS_OAUTH_EXPIRES_AT", (now_epoch_secs() + 3600).to_string());
        env
    }
}

impl Drop for DriftEnv {
    fn drop(&mut self) {
        restore_var("CS_CONFIG_DIR", self.config_dir.as_ref());
        restore_var("CS_ONPREM_URL", self.onprem_url.as_ref());
    }
}

fn restore_var(key: &str, value: Option<&OsString>) {
    match value {
        Some(value) => std::env::set_var(key, value),
        None => std::env::remove_var(key),
    }
}

fn signed_in(account: i64, token: &str) -> String {
    serde_json::json!({
        "status": "signed_in",
        "account-id": account,
        "access-token": token,
        "expires-at": now_epoch_secs() + 3600,
    })
    .to_string()
}

fn missing_slot() -> CliError {
    CliError::NonZeroExit {
        code: 1,
        stderr: NO_SLOT.to_string(),
    }
}

fn assert_token_checks(cli: &MockCliRunner, count: usize) {
    let calls = cli.calls();
    let calls = calls.lock().unwrap();
    assert_eq!(
        calls.as_slice(),
        vec![
            [
                "auth",
                "token",
                "--client",
                "mcp",
                "--output-format",
                "json"
            ];
            count
        ]
    );
}

async fn assert_drift_error(cli: &MockCliRunner, guidance: &str) {
    let Err(CredentialResolutionError::AccountDrift(message)) =
        AuthManager::new().resolve_credential(cli).await
    else {
        panic!("expected a Cloud account mismatch error");
    };
    for text in ["account 11", "account 3834", guidance] {
        assert!(message.contains(text), "missing {text:?} in {message:?}");
    }
}

#[tokio::test]
async fn recovers_to_cached_slot_and_persists_the_pin() {
    let _env = DriftEnv::new();
    let cli =
        MockCliRunner::with_responses(vec![Err(missing_slot()), Ok(signed_in(3834, "recovered"))]);
    let manager = AuthManager::new();
    for _ in 0..2 {
        let credential = manager.resolve_credential(&cli).await.unwrap().unwrap();
        assert_eq!(credential.access_token(), "recovered");
    }
    assert_token_checks(&cli, 2);
    assert_eq!(std::env::var("CS_ACCOUNT_ID").unwrap(), "3834");
    let saved = load().unwrap();
    assert_eq!(saved.values["account_id"], "3834");
    assert_eq!(saved.values["oauth_account_id"], "3834");
    assert_eq!(saved.values["oauth_token"], "recovered");
}

#[tokio::test]
async fn keeps_the_pin_if_its_slot_is_signed_in() {
    let _env = DriftEnv::new();
    let cli = MockCliRunner::with_ok(&signed_in(11, "pinned"));
    let credential = AuthManager::new()
        .resolve_credential(&cli)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(credential.access_token(), "pinned");
    assert_token_checks(&cli, 1);
    assert_eq!(std::env::var("CS_ACCOUNT_ID").unwrap(), "11");
    assert_eq!(load().unwrap().values["oauth_account_id"], "11");
}

#[tokio::test]
async fn refuses_to_fall_back_when_the_client_owns_the_pin() {
    let _env = DriftEnv::new();
    snapshot_client_env_vars();
    let cli = MockCliRunner::with_ok(SIGNED_OUT_JSON);
    assert_drift_error(&cli, "Update or remove CS_ACCOUNT_ID").await;
    assert_token_checks(&cli, 1);
    assert_eq!(std::env::var("CS_ACCOUNT_ID").unwrap(), "11");
}

#[tokio::test]
async fn refuses_a_recovery_slot_without_verified_credentials() {
    let env = DriftEnv::new();
    for response in [
        SIGNED_OUT_JSON.to_string(),
        signed_in(42, "wrong-slot"),
        signed_in(3834, " "),
    ] {
        let cli = MockCliRunner::with_responses(vec![Err(missing_slot()), Ok(response)]);
        assert_drift_error(&cli, "Neither account has a usable stored CLI session").await;
        assert_token_checks(&cli, 2);
        assert_eq!(std::env::var("CS_ACCOUNT_ID").unwrap(), "11");
        assert!(!env.dir.path().join("config.json").exists());
    }
}

#[tokio::test]
async fn refuses_to_switch_on_cli_errors_or_invalid_json() {
    let _env = DriftEnv::new();
    for response in [
        Ok("not json".to_string()),
        Err(CliError::NonZeroExit {
            code: 1,
            stderr: String::new(),
        }),
        Err(CliError::LicenseCheckFailed {
            stderr: "unavailable".to_string(),
        }),
    ] {
        let cli = MockCliRunner::with_responses(vec![response]);
        assert_drift_error(&cli, "Could not check the stored CLI session").await;
        assert_token_checks(&cli, 1);
    }
}

#[tokio::test]
async fn reports_a_recovery_config_write_failure() {
    let env = DriftEnv::new();
    let blocker = env.dir.path().join("not-a-directory");
    std::fs::write(&blocker, "").unwrap();
    std::env::set_var("CS_CONFIG_DIR", blocker);
    let cli =
        MockCliRunner::with_responses(vec![Err(missing_slot()), Ok(signed_in(3834, "recovered"))]);
    assert_drift_error(
        &cli,
        "Could not save the recovered session to MCP configuration",
    )
    .await;
    assert_token_checks(&cli, 2);
}

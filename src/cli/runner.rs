use std::path::Path;

use super::run_cli;
use crate::errors::CliError;

/// Trait abstracting CLI subprocess execution for dependency injection.
#[async_trait::async_trait]
pub trait CliRunner: Send + Sync {
    async fn run(&self, args: &[&str], working_dir: Option<&Path>) -> Result<String, CliError>;

    /// Run in a numbered OAuth slot without changing the configured account.
    async fn run_for_account(&self, args: &[&str], _account_id: i64) -> Result<String, CliError> {
        self.run(args, None).await
    }
}

pub struct ProductionCliRunner;

#[async_trait::async_trait]
impl CliRunner for ProductionCliRunner {
    async fn run(&self, args: &[&str], working_dir: Option<&Path>) -> Result<String, CliError> {
        run_cli(args, working_dir, None).await
    }

    async fn run_for_account(&self, args: &[&str], account_id: i64) -> Result<String, CliError> {
        run_cli(args, None, Some(account_id)).await
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[tokio::test]
    async fn account_override_is_scoped_to_one_cli_invocation() {
        let _env = crate::test_utils::clear_token();
        let dir = tempfile::tempdir().unwrap();
        let cli = dir.path().join("print-account");
        std::fs::write(&cli, "#!/bin/sh\nprintf '%s' \"$CS_ACCOUNT_ID\"\n").unwrap();
        std::fs::set_permissions(&cli, std::fs::Permissions::from_mode(0o700)).unwrap();

        let previous_path = std::env::var_os("CS_CLI_PATH");
        std::env::set_var("CS_CLI_PATH", cli);
        std::env::set_var("CS_ACCOUNT_ID", "11");
        let runner = ProductionCliRunner;
        let scoped = runner.run_for_account(&["auth", "token"], 3834).await;
        let normal = runner.run(&["review"], None).await;
        match previous_path {
            Some(path) => std::env::set_var("CS_CLI_PATH", path),
            None => std::env::remove_var("CS_CLI_PATH"),
        }

        assert_eq!(scoped.unwrap(), "3834");
        assert_eq!(normal.unwrap(), "11");
        assert_eq!(std::env::var("CS_ACCOUNT_ID").unwrap(), "11");
    }
}

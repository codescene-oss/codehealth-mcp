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

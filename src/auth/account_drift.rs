use crate::cli::CliRunner;
use crate::config::{is_client_env_var, ConfigEnvWriteGuard};

use super::{fetch_token_for_account, state, CliTokenResponse};

#[cfg(test)]
mod tests;

pub(super) struct AccountDrift {
    pin: i64,
    cached: i64,
}

pub(super) fn detect(guard: &ConfigEnvWriteGuard) -> Option<AccountDrift> {
    if state::guard_is_signed_out(guard) {
        return None;
    }
    let (pin, cached) = state::mismatched_accounts(
        guard.read_env("CS_ONPREM_URL").as_deref(),
        guard.read_env("CS_ACCOUNT_ID").as_deref(),
        guard.read_env("CS_OAUTH_ACCOUNT_ID").as_deref(),
    )?;
    Some(AccountDrift { pin, cached })
}

impl AccountDrift {
    pub(super) async fn recover(
        &self,
        cli_runner: &dyn CliRunner,
        guard: &ConfigEnvWriteGuard,
    ) -> Result<CliTokenResponse, String> {
        let (response, new_pin) = self.select_session(cli_runner).await?;
        state::persist_session(guard, &response, new_pin).map_err(|error| {
            tracing::warn!(%error, "could not persist reconciled OAuth session");
            self.error("Could not save the recovered session to MCP configuration.")
        })?;
        tracing::info!(
            pin = self.pin,
            cached = self.cached,
            ?new_pin,
            "reconciled Cloud OAuth session and CLI slot"
        );
        Ok(response)
    }

    async fn select_session(
        &self,
        cli_runner: &dyn CliRunner,
    ) -> Result<(CliTokenResponse, Option<i64>), String> {
        if let Some(response) = self.export(cli_runner, self.pin).await? {
            return Ok((response, None));
        }
        if is_client_env_var("CS_ACCOUNT_ID") {
            return Err(self.error("Update or remove CS_ACCOUNT_ID in your MCP client configuration and restart the server."));
        }
        let response = self.export(cli_runner, self.cached).await?.ok_or_else(|| {
            self.error("Neither account has a usable stored CLI session. Use `switch_account` to authorize the intended account.")
        })?;
        Ok((response, Some(self.cached)))
    }

    async fn export(
        &self,
        cli_runner: &dyn CliRunner,
        account_id: i64,
    ) -> Result<Option<CliTokenResponse>, String> {
        let response = fetch_token_for_account(cli_runner, account_id)
            .await
            .map_err(|_| self.error("Could not check the stored CLI session. Retry or use `switch_account` to authorize the intended account."))?;
        Ok(response.filter(|response| {
            response.account_id == Some(account_id)
                && super::credential_from_response(response).is_some()
        }))
    }

    fn error(&self, detail: &str) -> String {
        format!(
            "CodeScene Cloud account mismatch: the CLI is pinned to account {}, but the MCP OAuth session is for account {}. {detail}",
            self.pin, self.cached
        )
    }
}

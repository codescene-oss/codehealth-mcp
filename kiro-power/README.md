# CodeScene Code Health — Kiro power

A [Kiro power](https://kiro.dev/docs/powers/) in the [Agent Plugins](https://agent-plugins.org) format. It bundles:

- `mcp.json`: the [CodeHealth MCP Server](https://github.com/codescene-oss/codehealth-mcp), launched with `npx -y @codescene/codehealth-mcp` (requires Node.js 18 or later). Kiro manages and namespaces the server; you do not need to edit `~/.kiro/settings/mcp.json`.
- `skills/`: the public CodeHealth MCP skills for safeguarding AI-generated code, guiding refactoring, prioritizing technical debt, code ownership, risk-based testing, and configuration.

## Install

In Kiro IDE, open the Powers panel and choose **Add Custom Power**:

- **Import power from GitHub:** enter `https://github.com/codescene-oss/codehealth-mcp/tree/main/kiro-power`.
- **Import power from a folder:** select this `kiro-power` directory in a local clone.

## Sign in

Ask Kiro to *"Log me in to CodeScene"*. It calls the server's `login` tool and opens the browser sign-in flow. For standalone licenses, personal access tokens, or on-prem instances, see [Authentication](../docs/authentication.md).

## Maintenance

`skills/` and the `version` in `plugin.json` are synced from the repository root by the `update-packaging-metadata` workflow. Edit skills in [`/skills`](../skills/), not here.

use zed_extension_api::{self as zed, Result};

struct RestClientExtension {}

impl zed::Extension for RestClientExtension {
    fn new() -> Self {
        Self {}
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        if let Some(path) = worktree.which("http-lsp") {
            return Ok(zed::Command {
                command: path,
                args: vec![],
                env: Default::default(),
            });
        }

        let home = worktree
            .shell_env()
            .iter()
            .find(|(k, _)| k == "HOME")
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| "/root".to_string());

        let lsp_path = format!("{}/.local/bin/http-lsp", home);

        Ok(zed::Command {
            command: lsp_path,
            args: vec![],
            env: Default::default(),
        })
    }

    fn language_server_initialization_options(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        _worktree: &zed::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        Ok(None)
    }

    fn language_server_workspace_configuration(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        _worktree: &zed::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        Ok(None)
    }
}

zed::register_extension!(RestClientExtension);

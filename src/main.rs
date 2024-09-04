pub mod log;
pub mod download;

use std::path::PathBuf;
use anyhow::{anyhow, Result};
use lapce_plugin::{psp_types::{
    lsp_types::{request::Initialize, InitializeParams, MessageType},
    Request,
}, register_plugin, LapcePlugin, PLUGIN_RPC, VoltEnvironment};
use lapce_plugin::psp_types::lsp_types::Url;
use serde_json::Value;
use download::download_into;

#[derive(Default)]
struct State {}

register_plugin!(State);

impl LapcePlugin for State {
    fn handle_request(&mut self, _id: u64, method: String, params: Value) {
        #[allow(clippy::single_match)]
        match method.as_str() {
            Initialize::METHOD => {
                let params: InitializeParams = serde_json::from_value(params).unwrap();
                if let Err(e) = initialize(params) {
                    PLUGIN_RPC.window_show_message(
                        MessageType::ERROR,
                        format!("lldb-win couldn't be inited: {:?}", e),
                    ).unwrap();
                } else {
                    info!("lldb-win initialization succeeded");
                }
            }
            _ => {}
        }
    }
}

fn initialize(params: InitializeParams) -> Result<()> {
    let debugger_path = params
        .initialization_options
        .as_ref()
        .and_then(|options| options.get("debuggerPath"))
        .and_then(|server_path| server_path.as_str())
        .and_then(|server_path| {
            if !server_path.is_empty() {
                Some(server_path)
            } else {
                None
            }
        });

    if let Some(debugger_path) = debugger_path {
        info!("debugger_path: {}", debugger_path);
        PLUGIN_RPC.register_debugger_type("lldb".to_string(), debugger_path.to_string(), None)?;
        return Ok(());
    }

    if !PathBuf::from("lldb/adapter/codelldb.exe").exists() {
        let mut rs: Result<()> = Ok(());
        for _ in 0..3 {
            rs = download_into(PathBuf::from("lldb"));
            if rs.is_ok() {
                break;
            }
        }
        rs?;
    }
    // info!("env {:?}", std::env::vars());
    // let path = Url::parse(&VoltEnvironment::uri()?)?.to_file_path().map_err(|_| anyhow::format_err!("unable to parse url from file path"))?;
    // info!("Url {:?}", path);
    let path = Url::parse(&VoltEnvironment::uri()?)?
        .join("lldb/adapter/codelldb.exe")
        .map_err(|_| anyhow::format_err!("unable to parse url from file path"))?.to_file_path().and_then(|x| x.to_str().map(|x| x.to_string()).ok_or(())).map_err(|_|anyhow!("parse path fail"))?;

    PLUGIN_RPC.register_debugger_type("lldb".to_string(), path.split_at(1).1.to_string(), None)?;
    Ok(())
}

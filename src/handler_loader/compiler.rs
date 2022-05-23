use std::path::PathBuf;

use anyhow::Context;

use crate::wasm_module::WasmModuleSource;

use super::{
    loader::{LoadedHandlerConfiguration, LoadedHandlerConfigurationEntry},
    WasmHandlerConfiguration, WasmHandlerConfigurationEntry, HandlerInfo,
};

pub struct WasmCompilationSettings {
    pub cache_config_path: PathBuf,
}

pub fn compile(
    uncompiled_handlers: LoadedHandlerConfiguration,
    compilation_settings: WasmCompilationSettings,
) -> anyhow::Result<WasmHandlerConfiguration> {
    uncompiled_handlers.compile_modules(|module_bytes, info| {
        crate::wasm_module::WasmModuleSource::from_module_bytes(
            module_bytes,
            &compilation_settings.cache_config_path,
            info.allowed_hosts.clone(),
            info.http_max_concurrency,
        )
    })
}

impl LoadedHandlerConfiguration {
    pub fn compile_modules(
        self,
        compile: impl Fn(std::sync::Arc<Vec<u8>>, &HandlerInfo) -> anyhow::Result<WasmModuleSource>,
    ) -> anyhow::Result<WasmHandlerConfiguration> {
        let result: anyhow::Result<Vec<WasmHandlerConfigurationEntry>> = self
            .entries
            .into_iter()
            .map(|e| e.compile_module(|m, info| compile(m, &info)))
            .collect();
        Ok(WasmHandlerConfiguration { entries: result? })
    }
}

impl LoadedHandlerConfigurationEntry {
    pub fn compile_module(
        self,
        compile: impl Fn(std::sync::Arc<Vec<u8>>, &HandlerInfo) -> anyhow::Result<WasmModuleSource>,
    ) -> anyhow::Result<WasmHandlerConfigurationEntry> {
        let compiled_module = compile(self.module, &self.info)
            .with_context(|| format!("Error compiling Wasm module {}", &self.info.name))?;
        Ok(WasmHandlerConfigurationEntry {
            info: self.info,
            module: compiled_module,
        })
    }
}

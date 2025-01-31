use crate::ctx::EmbeddingCtx;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use anyhow::{bail, Result};
use wasmtime::component::Resource;
use wasmtime_wasi_io::{
    streams::{DynInputStream, DynOutputStream},
    IoView,
};

use super::wasi::cli::{environment, exit, stderr, stdin, stdout};

impl environment::Host for EmbeddingCtx {
    fn get_arguments(&mut self) -> Result<Vec<String>> {
        Ok(Vec::new())
    }
    fn get_environment(&mut self) -> Result<Vec<(String, String)>> {
        Ok(Vec::new())
    }
    fn initial_cwd(&mut self) -> Result<Option<String>> {
        Ok(None)
    }
}

impl exit::Host for EmbeddingCtx {
    fn exit(&mut self, code: Result<(), ()>) -> Result<()> {
        if code.is_ok() {
            bail!("wasi exit success")
        } else {
            bail!("wasi exit error")
        }
    }
}

impl stdin::Host for EmbeddingCtx {
    fn get_stdin(&mut self) -> Result<Resource<DynInputStream>> {
        let stdin: DynInputStream = Box::new(self.stdin());
        Ok(self.table().push(stdin)?)
    }
}

impl stdout::Host for EmbeddingCtx {
    fn get_stdout(&mut self) -> Result<Resource<DynOutputStream>> {
        let stdout: DynOutputStream = Box::new(self.stdout());
        Ok(self.table().push(stdout)?)
    }
}

impl stderr::Host for EmbeddingCtx {
    fn get_stderr(&mut self) -> Result<Resource<DynOutputStream>> {
        let stderr: DynOutputStream = Box::new(self.stderr());
        Ok(self.table().push(stderr)?)
    }
}

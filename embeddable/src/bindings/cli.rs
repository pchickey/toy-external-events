use crate::{EImpl, Embedding};
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

impl<E: Embedding> environment::Host for EImpl<E> {
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

impl<E: Embedding> exit::Host for EImpl<E> {
    fn exit(&mut self, code: Result<(), ()>) -> Result<()> {
        if code.is_ok() {
            bail!("wasi exit success")
        } else {
            bail!("wasi exit error")
        }
    }
}

impl<E: Embedding> stdin::Host for EImpl<E> {
    fn get_stdin(&mut self) -> Result<Resource<DynInputStream>> {
        let stdin: DynInputStream = Box::new(self.stdin());
        Ok(self.table().push(stdin)?)
    }
}

impl<E: Embedding> stdout::Host for EImpl<E> {
    fn get_stdout(&mut self) -> Result<Resource<DynOutputStream>> {
        let stdout: DynOutputStream = Box::new(self.stdout());
        Ok(self.table().push(stdout)?)
    }
}

impl<E: Embedding> stderr::Host for EImpl<E> {
    fn get_stderr(&mut self) -> Result<Resource<DynOutputStream>> {
        let stderr: DynOutputStream = Box::new(self.stderr());
        Ok(self.table().push(stderr)?)
    }
}

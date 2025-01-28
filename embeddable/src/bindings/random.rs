use crate::{EImpl, Embedding};
use alloc::vec::Vec;
use anyhow::Result;

use super::wasi::random::random;

impl<E: Embedding> random::Host for EImpl<E> {
    fn get_random_bytes(&mut self, len: u64) -> Result<Vec<u8>> {
        let mut vec = Vec::new();
        vec.resize(len as usize, 0u8);
        Ok(vec)
    }
    fn get_random_u64(&mut self) -> Result<u64> {
        Ok(0)
    }
}

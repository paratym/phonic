use crate::Sample;
use std::ops::{Deref, DerefMut};

pub const DEFAULT_BUF_LEN: usize = 4096;

pub struct DefaultBuf<S: Sample>([S; DEFAULT_BUF_LEN]);

impl<S: Sample> Default for DefaultBuf<S> {
    fn default() -> Self {
        Self([S::ORIGIN; DEFAULT_BUF_LEN])
    }
}

impl<S: Sample> Deref for DefaultBuf<S> {
    type Target = [S];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S: Sample> DerefMut for DefaultBuf<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

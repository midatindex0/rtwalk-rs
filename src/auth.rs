use send_wrapper::SendWrapper;
use std::ops::Deref;

#[derive(Clone, Debug)]
pub struct Shared<T>(SendWrapper<T>);

impl<T> Shared<T> {
    pub fn new(v: T) -> Self {
        Self(SendWrapper::new(v))
    }
}

impl<T> Deref for Shared<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub type SharedSession = Shared<actix_session::Session>;
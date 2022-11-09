#[derive(Debug)]
pub enum Error {
    FailedToGetStorage,
    FailedToDecode,
}

pub type Result<T> = core::result::Result<T, Error>;

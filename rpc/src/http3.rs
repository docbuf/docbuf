use crate::error::Error;

pub struct Http3Config<T>(T);

impl Http3Config<quiche::h3::Config> {
    pub fn new() -> Result<Self, Error> {
        Ok(Self(quiche::h3::Config::new()?))
    }
}

#[derive(Debug)]
pub enum Status {
    ResourceUnavailable,
    InvalidArguments,
    Internal,
}

impl std::error::Error for Status {}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Status::ResourceUnavailable => write!(f, "Resource Unavailable"),
            Status::Internal => write!(f, "Internal Error"),
            Status::InvalidArguments => write!(f, "Invalid Arguments"),
        }
    }
}

impl From<crate::Error> for Status {
    fn from(_value: crate::Error) -> Self {
        Status::Internal
    }
}

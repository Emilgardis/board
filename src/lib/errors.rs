

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("File is not currently supported")]
    NotSupported,
    #[error("Couldn't parse MoveIndex string")]
    MoveIndexParseError,
    #[error("Version {majv}.{minv} is not supported")]
    VersionNotSupported { majv: u8, minv: u8 },
    #[error("unsuccessful parsing of file in pos format")]
    PosParseError,
    #[error("unsuccessful parsing of file in RenLib format")]
    LibParseError,
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("{0}")]
    Other(String)
}


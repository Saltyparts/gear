use tobj::LoadError;

#[derive(Debug)]
pub enum GearError {
    IOError(std::io::Error),
    NetworkError(laminar::ErrorKind),
    OpenFileFailed,
    ParseFileFailed,
    Unknown,
}

impl From<LoadError> for GearError {
    fn from(e: LoadError) -> Self {
        match e {
            LoadError::OpenFileFailed => GearError::OpenFileFailed,
            LoadError::ReadError |
            LoadError::UnrecognizedCharacter |
            LoadError::PositionParseError |
            LoadError::NormalParseError |
            LoadError::TexcoordParseError |
            LoadError::FaceParseError |
            LoadError::MaterialParseError |
            LoadError::InvalidObjectName |
            LoadError::FaceVertexOutOfBounds |
            LoadError::FaceTexCoordOutOfBounds |
            LoadError::FaceNormalOutOfBounds |
            LoadError::FaceColorOutOfBounds => GearError::ParseFileFailed,
            LoadError::InvalidLoadOptionConfig |
            LoadError::GenericFailure => GearError::Unknown,
        }
    }
}

impl From<std::io::Error> for GearError {
    fn from(e: std::io::Error) -> Self {
        GearError::IOError(e)
    }
}

impl From<laminar::ErrorKind> for GearError {
    fn from(e: laminar::ErrorKind) -> Self {
        match e {
            laminar::ErrorKind::IOError(e) => GearError::IOError(e),
            _ => GearError::NetworkError(e),
        }
    }
}

pub type Result<T> = std::result::Result<T, GearError>;

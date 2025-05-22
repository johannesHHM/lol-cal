use core::fmt;

#[derive(Debug)]
pub enum Error {
    File(std::io::Error),
    NoConfigFile(String),
    EmptyHeader(usize),
    IncompleteHeader(usize),
    EmptyKey(usize),
    EmptyValue(usize),
    MissingSeperator(usize),
    InvalidKeybind(String),
    InvalidCommand(String),
    InvalidColor(String),
    InvalidBorder(String),
    InvalidBool(String),
    InvalidValue(String),
    UnknownKey(String, String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::File(e) => write!(f, "Config parsing error: {}", e),
            Error::NoConfigFile(path) => {
                write!(
                    f,
                    "Config parsing error: config file does not exist at {}",
                    path
                )
            }
            Error::EmptyHeader(line) => {
                write!(
                    f,
                    "Config parsing error: empty header on line {}. Header must have name",
                    line
                )
            }
            Error::IncompleteHeader(line) => {
                write!(
                    f,
                    "Config parsing error: incomplete header on line {}. Header must be closed by square brackets",
                    line
                )
            }
            Error::EmptyKey(line) => {
                write!(
                    f,
                    "Config parsing error: empty key on line {}. Entry must have a key value pair",
                    line
                )
            }
            Error::EmptyValue(line) => {
                write!(
                    f,
                    "Config parsing error: empty value on line {}. Entry must have a key value pair",
                    line
                )
            }
            Error::MissingSeperator(line) => {
                write!(
                    f,
                    "Config parsing error: missing seperator on line {}. Entry must have a seperator",
                    line
                )
            }
            Error::InvalidKeybind(raw) => {
                write!(
                    f,
                    "Config parsing error: unable to parse key from '{}'",
                    raw
                )
            }
            Error::InvalidCommand(raw) => {
                write!(
                    f,
                    "Config parsing error: unable to parse command from '{}'",
                    raw
                )
            }
            Error::InvalidColor(raw) => {
                write!(
                    f,
                    "Config parsing error: unable to parse style from '{}'",
                    raw
                )
            }
            Error::InvalidBorder(raw) => {
                write!(
                    f,
                    "Config parsing error: unable to parse border from '{}'",
                    raw
                )
            }
            Error::InvalidBool(raw) => {
                write!(
                    f,
                    "Config parsing error: unable to parse boolean value from '{}'",
                    raw
                )
            }
            Error::InvalidValue(raw) => {
                write!(
                    f,
                    "Config parsing error: unable to parse value from '{}'",
                    raw
                )
            }
            Error::UnknownKey(raw_key, subsection) => {
                write!(
                    f,
                    "Config parsing error: unknown key '{}' for section '{}'",
                    raw_key, subsection
                )
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::File(e) => Some(e),
            Error::NoConfigFile(_) => None,
            Error::EmptyHeader(_) => None,
            Error::IncompleteHeader(_) => None,
            Error::EmptyKey(_) => None,
            Error::EmptyValue(_) => None,
            Error::MissingSeperator(_) => None,
            Error::InvalidKeybind(_) => None,
            Error::InvalidCommand(_) => None,
            Error::InvalidColor(_) => None,
            Error::InvalidBorder(_) => None,
            Error::InvalidBool(_) => None,
            Error::InvalidValue(_) => None,
            Error::UnknownKey(_, _) => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::File(error)
    }
}

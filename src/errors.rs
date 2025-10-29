#[derive(Debug)]
pub enum WmError {
    X11(X11Error),
    Io(std::io::Error),
    Anyhow(anyhow::Error),
    Config(ConfigError),
}

#[derive(Debug)]
pub enum X11Error {
    ConnectError(x11rb::errors::ConnectError),
    ConnectionError(x11rb::errors::ConnectionError),
    ReplyError(x11rb::errors::ReplyError),
    ReplyOrIdError(x11rb::errors::ReplyOrIdError),
    DisplayOpenFailed,
    FontLoadFailed(String),
    DrawCreateFailed,
}

#[derive(Debug)]
pub enum ConfigError {
    ParseError(ron::error::SpannedError),
    InvalidModkey(String),
    UnknownKey(String),
    UnknownAction(String),
    UnknownBlockCommand(String),
    MissingCommandArg { command: String, field: String },
    InvalidVariableName(String),
    InvalidDefine(String),
    UndefinedVariable(String),
}

impl std::fmt::Display for WmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::X11(error) => write!(f, "{}", error),
            Self::Io(error) => write!(f, "{}", error),
            Self::Anyhow(error) => write!(f, "{}", error),
            Self::Config(error) => write!(f, "{}", error),
        }
    }
}

impl std::error::Error for WmError {}

impl std::fmt::Display for X11Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConnectError(err) => write!(f, "{}", err),
            Self::ConnectionError(err) => write!(f, "{}", err),
            Self::ReplyError(err) => write!(f, "{}", err),
            Self::ReplyOrIdError(err) => write!(f, "{}", err),
            Self::DisplayOpenFailed => write!(f, "failed to open X11 display"),
            Self::FontLoadFailed(font_name) => write!(f, "failed to load Xft font: {}", font_name),
            Self::DrawCreateFailed => write!(f, "failed to create XftDraw"),
        }
    }
}

impl std::error::Error for X11Error {}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseError(err) => write!(f, "Failed to parse RON config: {}", err),
            Self::InvalidModkey(key) => write!(f, "Invalid modkey: {}", key),
            Self::UnknownKey(key) => write!(f, "Unknown key: {}", key),
            Self::UnknownAction(action) => write!(f, "Unknown action: {}", action),
            Self::UnknownBlockCommand(cmd) => write!(f, "Unknown block command: {}", cmd),
            Self::MissingCommandArg { command, field } => {
                write!(f, "{} command requires {}", command, field)
            }
            Self::InvalidVariableName(name) => {
                write!(f, "Invalid variable name '{}': must start with $", name)
            }
            Self::InvalidDefine(line) => {
                write!(f, "Invalid #DEFINE syntax: '{}'. Expected: #DEFINE $var_name = value", line)
            }
            Self::UndefinedVariable(var) => {
                write!(f, "Undefined variable '{}': define it with #DEFINE before use", var)
            }
        }
    }
}

impl std::error::Error for ConfigError {}

impl<T: Into<X11Error>> From<T> for WmError {
    fn from(value: T) -> Self {
        Self::X11(value.into())
    }
}

impl From<std::io::Error> for WmError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<anyhow::Error> for WmError {
    fn from(value: anyhow::Error) -> Self {
        Self::Anyhow(value)
    }
}

impl From<ConfigError> for WmError {
    fn from(value: ConfigError) -> Self {
        Self::Config(value)
    }
}

impl From<ron::error::SpannedError> for ConfigError {
    fn from(value: ron::error::SpannedError) -> Self {
        ConfigError::ParseError(value)
    }
}

impl From<x11rb::errors::ConnectError> for X11Error {
    fn from(value: x11rb::errors::ConnectError) -> Self {
        X11Error::ConnectError(value)
    }
}

impl From<x11rb::errors::ConnectionError> for X11Error {
    fn from(value: x11rb::errors::ConnectionError) -> Self {
        X11Error::ConnectionError(value)
    }
}

impl From<x11rb::errors::ReplyError> for X11Error {
    fn from(value: x11rb::errors::ReplyError) -> Self {
        X11Error::ReplyError(value)
    }
}

impl From<x11rb::errors::ReplyOrIdError> for X11Error {
    fn from(value: x11rb::errors::ReplyOrIdError) -> Self {
        X11Error::ReplyOrIdError(value)
    }
}

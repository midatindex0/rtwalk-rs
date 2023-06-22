#[non_exhaustive]
#[derive(Debug)]
pub enum UserCreationError<'a> {
    // Username errors
    InvalidUsername(&'a str),
    UsernameAlreadyExists(&'a str),
    ReservedUsername(&'a str),
    // Password errors
    PasswordTooShort(&'a str),
    PasswordTooLong(&'a str),
    LowPasswordStrength(&'a str),
    // Db errors
    InternalError(&'a str),
}

impl<'a> std::error::Error for UserCreationError<'a> {}

impl<'a> std::fmt::Display for UserCreationError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            UserCreationError::InvalidUsername(m) => write!(f, "{}", m),
            UserCreationError::UsernameAlreadyExists(m) => write!(f, "{}", m),
            UserCreationError::ReservedUsername(m) => write!(f, "{}", m),
            UserCreationError::PasswordTooShort(m) => write!(f, "{}", m),
            UserCreationError::PasswordTooLong(m) => write!(f, "{}", m),
            UserCreationError::LowPasswordStrength(m) => write!(f, "{}", m),
            UserCreationError::InternalError(m) => write!(f, "{}", m),
        }
    }
}

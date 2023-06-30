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

#[non_exhaustive]
#[derive(Debug)]
pub enum ForumCreationError<'a> {
    // Forum Name Errors
    InvalidName(&'a str),
    ForumAlreadyExists(&'a str),
    ReservedName(&'a str),
    // Db errors
    InternalError(&'a str),
}

impl<'a> std::error::Error for ForumCreationError<'a> {}

impl<'a> std::fmt::Display for ForumCreationError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ForumCreationError::InvalidName(m) => write!(f, "{}", m),
            ForumCreationError::ForumAlreadyExists(m) => write!(f, "{}", m),
            ForumCreationError::ReservedName(m) => write!(f, "{}", m),
            ForumCreationError::InternalError(m) => write!(f, "{}", m),
        }
    }
}

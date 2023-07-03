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
            Self::InvalidUsername(m) => write!(f, "{}", m),
            Self::UsernameAlreadyExists(m) => write!(f, "{}", m),
            Self::ReservedUsername(m) => write!(f, "{}", m),
            Self::PasswordTooShort(m) => write!(f, "{}", m),
            Self::PasswordTooLong(m) => write!(f, "{}", m),
            Self::LowPasswordStrength(m) => write!(f, "{}", m),
            Self::InternalError(m) => write!(f, "{}", m),
        }
    }
}

#[non_exhaustive]
#[derive(Debug)]
pub enum UserAuthError<'a> {
    InvalidUsernameOrPassword(&'a str),
    UserNotFound(&'a str),
    // Db errors
    InternalError(&'a str),
}

impl<'a> std::error::Error for UserAuthError<'a> {}

impl<'a> std::fmt::Display for UserAuthError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::InvalidUsernameOrPassword(m) => write!(f, "{}", m),
            Self::UserNotFound(m) => write!(f, "{}", m),
            Self::InternalError(m) => write!(f, "{}", m),
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
            Self::InvalidName(m) => write!(f, "{}", m),
            Self::ForumAlreadyExists(m) => write!(f, "{}", m),
            Self::ReservedName(m) => write!(f, "{}", m),
            Self::InternalError(m) => write!(f, "{}", m),
        }
    }
}

#[non_exhaustive]
#[derive(Debug)]
pub enum PostCreationError<'a> {
    InternalError(&'a str),
    ForumNotFound(&'a str),
}

impl<'a> std::error::Error for PostCreationError<'a> {}

impl<'a> std::fmt::Display for PostCreationError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::InternalError(m) => write!(f, "{}", m),
            Self::ForumNotFound(m) => write!(f, "{}", m),
        }
    }
}

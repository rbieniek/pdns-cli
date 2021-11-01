use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Error {
    error: String,
    errors: Option<Vec<String>>,
}

impl Error {
    pub fn error(&self) -> String {
        self.error.clone()
    }

    pub fn errors(&self) -> Option<Vec<String>> {
        self.errors.clone()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.errors {
            Some(errors) => write!(f, "error: {}, errors: {}",
                                           self.error.clone(),
                                           errors.join(", ")),
            None => write!(f, "error: {}",
                           self.error.clone()),
        }
    }
}
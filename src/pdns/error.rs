// Copyright 2021 Cumulus Cloud Software und Consulting GmbH & Co KG
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Error {
    error: String,
    errors: Option<Vec<String>>,
}

#[allow(dead_code)]
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
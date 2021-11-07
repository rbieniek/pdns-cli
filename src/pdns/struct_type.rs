use serde::{Serialize, Deserialize};
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum StructType {
    None,
    Server,
    Zone,
}

impl Display for StructType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StructType::Server => write!(f, "SERVER"),
            StructType::Zone => write!(f, "ZONE"),
            StructType::None => write!(f, "(pseudo)NONE"),
        }
    }
}
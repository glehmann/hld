use clap::ValueEnum;

use crate::error::*;
use std::str::FromStr;

#[derive(Debug, Eq, PartialEq, Copy, ValueEnum)]
#[value(rename_all = "lowercase")]
pub enum Strategy {
    HardLink,
    SymLink,
    RefLink,
}

impl FromStr for Strategy {
    type Err = Error;
    fn from_str(value: &str) -> Result<Strategy> {
        let value = value.to_lowercase();
        if value == "hardlink" {
            Ok(Strategy::HardLink)
        } else if value == "symlink" {
            Ok(Strategy::SymLink)
        } else if value == "reflink" {
            Ok(Strategy::RefLink)
        } else {
            Err(crate::error::Error::Strategy { name: value })
        }
    }
}

impl std::fmt::Display for Strategy {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Strategy::HardLink => write!(f, "hardlink"),
            Strategy::SymLink => write!(f, "symlink"),
            Strategy::RefLink => write!(f, "reflink"),
        }
    }
}

impl Clone for Strategy {
    #[inline]
    fn clone(&self) -> Strategy {
        *self
    }
}

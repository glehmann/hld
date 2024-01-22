use clap::ValueEnum;
use strum::{Display, EnumString};

#[derive(Debug, Eq, PartialEq, Copy, ValueEnum, Clone, EnumString, Display)]
#[value(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum Strategy {
    HardLink,
    SymLink,
    RefLink,
}

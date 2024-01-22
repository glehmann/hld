use clap::ValueEnum;
use strum::Display;

#[derive(Debug, Eq, PartialEq, Copy, ValueEnum, Clone, Display)]
#[value(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum Strategy {
    HardLink,
    SymLink,
    RefLink,
}

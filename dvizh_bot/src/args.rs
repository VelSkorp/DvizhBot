use clap::{Parser, ValueEnum};

#[derive(Parser, Clone, Debug)]
pub struct Arguments {
    #[clap(value_enum, default_value_t=Verbose::Debug)]
    pub verbose: Verbose,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Verbose {
    Debug,
    Info,
    Warn,
    Error,
}
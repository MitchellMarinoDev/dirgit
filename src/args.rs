use clap::{Parser, ValueEnum};
use colored::control::{set_override, unset_override};

/// The options for the --color flag.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, ValueEnum)]
pub enum ColorOptions {
    #[default]
    Auto,
    Always,
    Never,
}

/// A cli tool that lets you quickly check the git status of all the current directory
/// (or subdirectories).
///
/// This lets you know if all your programs are backed up in git.
#[derive(Parser, Debug, Clone, Eq, PartialEq, Hash)]
#[command(author, version, about)]
pub struct Args {
    /// Flag to not fetch git repos
    #[arg(short, long)]
    pub no_fetch: bool,

    /// Flag for verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// The limit for how deep to look for git repos
    #[arg(short, long)]
    #[arg(default_value_t = 3)]
    pub recurse_limit: u32,

    /// How to output colors on the terminal
    #[arg(short, long)]
    #[arg(value_enum)]
    #[arg(default_value_t = ColorOptions::Auto)]
    pub color: ColorOptions,

    #[arg(default_value_t = String::from("."))]
    /// The path to the directory you are checking
    pub path: String,
}

impl Args {
    /// Applies the terminal colorizing settings from the `color` field.
    pub fn apply_color_option(&self) {
        match self.color {
            ColorOptions::Auto => unset_override(),
            ColorOptions::Always => set_override(true),
            ColorOptions::Never => set_override(false),
        }
    }
}

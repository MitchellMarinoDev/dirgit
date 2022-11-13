use clap::Parser;

/// A cli tool that lets you quickly check the git status of all the current directory
/// (or subdirectories).
///
/// This lets you know if all your programs are backed up in git.
#[derive(Parser, Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[command(author, version, about)]
pub struct Args {
    /// Flag for verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Flag to not fetch git repos
    #[arg(short, long)]
    pub no_fetch: bool,

    /// Weather to use color in the output
    #[arg(short, long, default_value_t = true)]
    pub color: bool,
}

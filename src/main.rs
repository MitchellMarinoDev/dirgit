use crate::issues::find_issues;
use clap::Parser;

mod args;
mod issues;

fn main() {
    let args = args::Args::parse();

    let issues = find_issues(args);
    println!("{}", issues.output(args));
}

use crate::issues::{find_issues, Issues};
use clap::Parser;

mod args;
mod issues;

fn main() {
    let args = args::Args::parse();
    args.apply_color_option();

    let mut issues = Issues::default();
    find_issues(&args, &mut issues, args.path.clone(), args.recurse_limit);
    println!("{}", issues.output(&args));
}

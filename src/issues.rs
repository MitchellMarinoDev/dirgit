use crate::args::Args;
use colored::Colorize;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default)]
pub struct Issues {
    dir_searched: i32,
    no_git_repo: Vec<String>,
    no_remote: Vec<String>,
    current_branch_untracked: Vec<String>,
    not_committed: Vec<String>,
    not_pushed: Vec<String>,
    have_diverged: Vec<String>,
}

impl Issues {
    pub fn output(&self, args: &Args) -> String {
        fn colorize_count(u: usize) -> String {
            if u > 0 {
                format!("{}", u.to_string().bold().red())
            } else {
                format!("{}", u.to_string().green())
            }
        }

        fn section(args: &Args, s: &mut String, title: &str, contents: &Vec<String>) {
            let count = contents.len();
            if !args.verbose && count == 0 {
                return;
            }

            let dots = ".".repeat(37 - title.len());
            s.push_str(&format!(
                "{} {} {}\n",
                title.bold().blue(),
                dots,
                colorize_count(count),
            ));

            for path in contents.iter() {
                s.push_str(&format!("    {}\n", path));
            }
        }

        let mut s = String::new();

        section(args, &mut s, "Non Git Repos", &self.no_git_repo);
        section(args, &mut s, "Repos with No Remote Origin", &self.no_remote);
        section(
            args,
            &mut s,
            "Repos with Current Branch Untracked",
            &self.current_branch_untracked,
        );
        section(
            args,
            &mut s,
            "Repos with Uncommitted Files",
            &self.not_committed,
        );
        section(
            args,
            &mut s,
            "Repos with Un-pushed Commits",
            &self.not_pushed,
        );
        section(
            args,
            &mut s,
            "Repos with Diverged Branches",
            &self.have_diverged,
        );

        if s.is_empty() {
            return "No issues found :)".bold().green().to_string();
        }
        s.trim_end().to_owned()
    }
}

pub fn find_issues(args: &Args, issues: &mut Issues, directory: String, recurse_limit: u32) {
    if recurse_limit < 1 {
        return;
    }

    if Path::exists(format!("{}/.git", directory).as_ref()) {
        find_issues_with(args, issues, directory);
    } else {
        let paths = match fs::read_dir(&directory) {
            Ok(paths) => paths,

            Err(e) => {
                eprintln!("Failed to read dir {}: {}", directory, e);
                return;
            }
        };

        for dir_entry in paths
            .filter_map(|p| p.ok())
            .filter(|p| p.metadata().map(|m| m.is_dir()).unwrap_or(false))
        {
            if let Some(path) = dir_entry.path().to_str() {
                find_issues(args, issues, path.to_owned(), recurse_limit - 1);
            }
        }
    }
}

fn find_issues_with(args: &Args, issues: &mut Issues, directory: String) {
    issues.dir_searched += 1;

    // perform git fetch
    if !args.no_fetch {
        Command::new("git")
            .arg("fetch")
            .current_dir(&directory)
            .output()
            .expect("`git fetch` command failed");
    }

    // check git status
    let git_status = Command::new("git")
        .arg("status")
        .current_dir(&directory)
        .output()
        .expect("`git status` command failed");

    // check for remote
    let git_remote = Command::new("git")
        .arg("remote")
        .current_dir(&directory)
        .output()
        .expect(&*format!(
            "`git remote` command failed on dir {}",
            directory
        ));

    // check if current branch is tracked
    let git_branch_vv = Command::new("git")
        .arg("branch")
        .arg("-vv")
        .arg("--color=never")
        .current_dir(&directory)
        .output()
        .expect(&*format!(
            "`git remote` command failed on dir {}",
            directory
        ));

    if git_status
        .stderr
        .starts_with(b"fatal: not a git repository")
    {
        return issues.no_git_repo.push(directory.clone());
    }

    if !is_sub(&git_remote.stdout, b"origin") {
        return issues.no_remote.push(directory.clone());
    }

    let git_branch_vv_out = String::from_utf8(git_branch_vv.stdout.clone())
        .expect("`git branch -vv` gave invalid utf-8");
    let current_branch = git_branch_vv_out
        .lines()
        .find_map(|l| {
            let mut words = l.split(" ");
            if words.next() == Some("*") {
                return words.next();
            }
            None
        })
        .expect("could not find current branch");
    if !is_sub(
        &git_branch_vv.stdout,
        format!("[origin/{}", current_branch).as_bytes(),
    ) {
        return issues.current_branch_untracked.push(directory.clone());
    }

    if is_sub(&git_status.stdout, b"Changes to be committed:")
        || is_sub(&git_status.stdout, b"Changes not staged for commit:")
        || is_sub(&git_status.stdout, b"Untracked files:")
    {
        return issues.not_committed.push(directory.clone());
    }

    if is_sub(&git_status.stdout, b"Your branch is ahead of") {
        return issues.not_pushed.push(directory.clone());
    }

    if is_sub(&git_status.stdout, b"have diverged") {
        return issues.have_diverged.push(directory.clone());
    }
}

fn is_sub<T: PartialEq>(haystack: &[T], needle: &[T]) -> bool {
    haystack.windows(needle.len()).any(|c| c == needle)
}

#[test]
fn test_is_sub() {
    // Should be true
    assert!(is_sub(b"Hello, world!", b"Hello"));
    assert!(is_sub(b"Hello, world!", b"Hello, world!"));
    assert!(is_sub(b"Hello, world!", b"ello"));
    assert!(is_sub(b"Hello, world!", b"llo, wor"));
    assert!(is_sub(b"Hello, world!", b"world!"));

    // Should be false
    assert!(!is_sub(b"Hello, world!", b"other"));
    assert!(!is_sub(b"Hello, world!", b"Hello, world! with more"));
    assert!(!is_sub(b"Hello, world!", b"Hello,  world!"));
    assert!(!is_sub(b"Hello, world!", b" Hello, world!"));
}

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
    pub fn output(&self, args: Args) -> String {
        if args.verbose {
            self.verbose(args)
        } else {
            self.overview(args)
        }
    }

    pub fn overview(&self, args: Args) -> String {
        fn colorize(u: usize) -> String {
            if u > 0 { format!("{}", u.to_string().bold().red()) } else { format!("{}", u.to_string().green()) }
        }

        let total = self.no_git_repo.len()
            + self.not_committed.len()
            + self.not_pushed.len()
            + self.have_diverged.len()
            + self.no_remote.len();

        if args.color {
            format!(
                "\
                {} ........................ {}\n\
                {} .......... {}\n\
                {} .. {}\n\
                {} ......... {}\n\
                {} ......... {}\n\
                {} ......... {}\n\
                {} ......................... {}/{}\
                ",
                "Non Git Repos".blue(),
                colorize(self.no_git_repo.len()),
                "Repos with No Remote Origin".blue(),
                colorize(self.no_remote.len()),
                "Repos with Current Branch Untracked".blue(),
                colorize(self.current_branch_untracked.len()),
                "Repos with Uncommitted Files".blue(),
                colorize(self.not_committed.len()),
                "Repos with Un-pushed Commits".blue(),
                colorize(self.not_pushed.len()),
                "Repos with Diverged Branches".blue(),
                colorize(self.have_diverged.len()),
                "Total Issues".blue(),
                colorize(total),
                self.dir_searched,
            )
        } else {
            format!(
                "\
                Non Git Repos ........................ {}\n\
                Repos with No Remote Origin .......... {}\n\
                Repos with Current Branch Untracked .. {}\n\
                Repos with Uncommitted Files ......... {}\n\
                Repos with Un-pushed Commits ......... {}\n\
                Repos with Diverged Branches ......... {}\n\
                Total Issues ......................... {}/{}\
                ",
                self.no_git_repo.len(),
                self.no_remote.len(),
                self.current_branch_untracked.len(),
                self.not_committed.len(),
                self.not_pushed.len(),
                self.have_diverged.len(),
                total,
                self.dir_searched,
            )
        }
    }

    pub fn verbose(&self, args: Args) -> String {
        fn section(s: &mut String, title: &str, contents: &Vec<String>) {
            let count = contents.len();
            if count == 0 {
                return;
            }

            s.push_str(&*format!(
                "{}",
                format!("{} ({}):\n", title, count).bold().blue()
            ));
            for path in contents.iter() {
                s.push_str(&*path);
                s.push('\n');
            }
        }

        let mut s = String::new();

        section(&mut s, "Non Git Repos", &self.no_git_repo);
        section(&mut s, "Repos with No Remote Origin", &self.no_remote);
        section(&mut s, "Repos with Current Branch Untracked", &self.current_branch_untracked);
        section(&mut s, "Repos with Uncommitted Files", &self.not_committed);
        section(&mut s, "Repos with Un-pushed Commits", &self.not_pushed);
        section(&mut s, "Repos with Diverged Branches", &self.have_diverged);

        s.push_str(&*self.overview(args));
        s
    }
}

pub fn find_issues(args: Args) -> Issues {
    let mut issues = Issues::default();

    if Path::exists("./.git".as_ref()) {
        find_issues_with("./".into(), &mut issues, args);
        return issues;
    }

    let paths = fs::read_dir("./").expect("could not read the current directory");
    for dir_entry in paths
        .filter_map(|p| p.ok())
        .filter(|p| p.metadata().map(|m| m.is_dir()).unwrap_or(false))
    {
        let path = dir_entry.path().to_str().unwrap_or("").to_owned();
        find_issues_with(path, &mut issues, args);
    }
    issues
}

fn find_issues_with(path: String, issues: &mut Issues, args: Args) {
    issues.dir_searched += 1;

    // perform git fetch
    if !args.no_fetch {
        Command::new("git")
            .arg("fetch")
            .current_dir(&path)
            .output()
            .expect("`git fetch` command failed");
    }

    // check git status
    let git_status = Command::new("git")
        .arg("status")
        .current_dir(&path)
        .output()
        .expect("`git status` command failed");

    // check for remote
    let git_remote = Command::new("git")
        .arg("remote")
        .current_dir(&path)
        .output()
        .expect(&*format!("`git remote` command failed on dir {}", path));

    // check if current branch is tracked
    let git_branch_vv = Command::new("git")
        .arg("branch")
        .arg("-vv")
        .arg("--color=never")
        .current_dir(&path)
        .output()
        .expect(&*format!("`git remote` command failed on dir {}", path));

    if git_status
        .stderr
        .starts_with(b"fatal: not a git repository")
    {
        return issues.no_git_repo.push(path.clone());
    }

    if !is_sub(&git_remote.stdout, b"origin") {
        return issues.no_remote.push(path.clone());
    }

    let git_branch_vv_out = String::from_utf8(git_branch_vv.stdout.clone()).expect("`git branch -vv` gave invalid utf-8");
    let current_branch = git_branch_vv_out.lines().filter_map(|l| {
        let mut words = l.split(" ");
        if words.next() == Some("*") {
            return words.next();
        }
        None
    }).next().expect("could not find current branch");
    if !is_sub(&git_branch_vv.stdout, format!("[origin/{}", current_branch).as_bytes()) {
        return issues.current_branch_untracked.push(path.clone());
    }

    if is_sub(&git_status.stdout, b"Changes to be committed:")
        || is_sub(&git_status.stdout, b"Changes not staged for commit:")
        || is_sub(&git_status.stdout, b"Untracked files:")
    {
        return issues.not_committed.push(path.clone());
    }

    if is_sub(&git_status.stdout, b"Your branch is ahead of") {
        return issues.not_pushed.push(path.clone());
    }

    if is_sub(&git_status.stdout, b"have diverged") {
        return issues.have_diverged.push(path.clone());
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

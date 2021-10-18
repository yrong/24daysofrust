extern crate git2;
extern crate chrono;

use git2::{Commit, Direction, ObjectType, Oid, Repository, Signature};
use std::fs::{canonicalize, File};
use std::io::Write;
use std::path::Path;
use chrono::prelude::*;
use chrono::offset::LocalResult;

fn find_last_commit(repo: &Repository) -> Result<Commit, git2::Error> {
    let obj = repo.head()?.resolve()?.peel(ObjectType::Commit)?;
    obj.into_commit()
        .map_err(|_| git2::Error::from_str("Couldn't find commit"))
}

fn display_commit(commit: &Commit) {
    let timestamp = commit.time().seconds();
    let tm = Utc.timestamp(timestamp,0);//time::at(time::Timespec::new(timestamp, 0));
    println!(
        "commit {}\nAuthor: {}\nDate:   {}\n\n    {}",
        commit.id(),
        commit.author(),
        tm.to_rfc2822(),
        commit.message().unwrap_or("no commit message")
    );
}

fn add_and_commit(repo: &Repository, path: &Path, message: &str) -> Result<Oid, git2::Error> {
    let mut index = repo.index()?;
    index.add_path(path)?;
    let oid = index.write_tree()?;
    let signature = Signature::now("Zbigniew Siciarz", "zbigniew@siciarz.net")?;
    let parent_commit = find_last_commit(repo)?;
    let tree = repo.find_tree(oid)?;
    repo.commit(
        Some("HEAD"), //  point HEAD to our new commit
        &signature,   // author
        &signature,   // committer
        message,      // commit message
        &tree,        // tree
        &[&parent_commit],
    ) // parents
}

fn push(repo: &Repository, url: &str) -> Result<(), git2::Error> {
    let mut remote = match repo.find_remote("yrong") {
        Ok(r) => r,
        Err(_) => repo.remote("yrong", url)?,
    };
    remote.connect(Direction::Push)?;
    remote.push(&["refs/heads/master:refs/heads/master"], None)
}

fn main() {
    println!("24 Days of Rust vol. 2 - git2");
    let repo_root = std::env::args().nth(1).unwrap_or(".".to_string());
    let repo = Repository::open(repo_root.as_str()).expect("Couldn't open repository");
    println!("{} state={:?}", repo.path().display(), repo.state());
    let commit = find_last_commit(&repo).expect("Couldn't find last commit");
    display_commit(&commit);

    let relative_path = Path::new("example.txt");
    {
        let file_path = Path::new(repo_root.as_str()).join(relative_path);
        let mut file = File::create(file_path.clone()).expect("Couldn't create file");
        file.write_all(b"Hello git2").unwrap();
    }
    let commit_id = add_and_commit(&repo, relative_path, "Add example text file")
        .expect("Couldn't add file to repo");
    println!("New commit: {}", commit_id);

    let remote_url = "git@github.com:yrong/24daysofrust.git";
    // format!(
    //     "file://{}",
    //     canonicalize(".git/config").unwrap().display()
    // );
    println!("Pushing to: {}", remote_url);
    // push(&repo, remote_url).expect("Couldn't push to remote repo");
}

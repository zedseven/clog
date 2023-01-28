//! The module for searching the repo for specific commit data.

// Uses
use std::{collections::HashSet, path::Path, process::Command};

use anyhow::{Context, Result};
use linked_hash_set::LinkedHashSet;

use crate::{
	collection::Commit,
	index::Index,
	util::{inside_out_result, run_command},
};

#[derive(Debug)]
pub struct SearchResults<'a> {
	pub included_commits:      Vec<IncludedCommit<'a>>,
	pub included_jira_tickets: Vec<&'a str>,
}

#[derive(Debug)]
pub struct IncludedCommit<'a> {
	pub commit:             &'a Commit,
	pub referenced_commits: Vec<IncludedCommit<'a>>,
}

pub fn get_search_results<'a, P>(
	index: &Index<'a>,
	repo_dir: P,
	revspec: &str,
	affected_filepaths: Option<&str>,
) -> Result<SearchResults<'a>>
where
	P: AsRef<Path>,
{
	dbg!(revspec);
	dbg!(affected_filepaths);

	// Prepare the `git log` command for the search
	let mut command = Command::new("git");
	command
		.arg("log")
		.arg("--pretty=format:%H") // Just the hashes
		.arg(revspec)
		.current_dir(repo_dir);
	if let Some(filepaths) = affected_filepaths {
		command.arg("--"); // This is necessary to separate the filepaths from the revspec/commits
		command.arg(filepaths);
	}

	// Run the command
	let commit_list = run_command(command).with_context(|| "unable to get the repo log")?;
	dbg!(&commit_list);

	// Process each commit and build a final list of all tickets, commits, merges,
	// etc.
	let mut visited_commits = HashSet::new();
	let mut jira_tickets_set = LinkedHashSet::new();

	let included_commits = commit_list
		.lines()
		.filter_map(|line| {
			let line = line.trim();
			(!line.is_empty()).then(|| {
				index
					.lookup_git_revision(line)
					.expect("all commits returned as search results should be in the index")
			})
		})
		.map(|commit| visit_commit(index, &mut visited_commits, &mut jira_tickets_set, commit))
		.filter_map(inside_out_result)
		.collect::<Result<Vec<_>>>()
		.with_context(|| "unable to process the commit search results")?;

	Ok(SearchResults {
		included_commits,
		included_jira_tickets: Vec::from_iter(jira_tickets_set),
	})
}

fn visit_commit<'a>(
	index: &Index<'a>,
	visited_commits: &mut HashSet<&'a str>,
	jira_tickets_set: &mut LinkedHashSet<&'a str>,
	commit: &'a Commit,
) -> Result<Option<IncludedCommit<'a>>> {
	dbg!(&visited_commits);
	dbg!(&jira_tickets_set);
	dbg!(&commit);

	// Store the commit in the visited list and check to ensure that it's new
	let visited_previously = !visited_commits.insert(commit.git_revision.as_str());
	if visited_previously {
		return Ok(None);
	}

	// Collect the Jira tickets
	jira_tickets_set.extend(commit.jira_tickets.iter().map(String::as_str));

	// Prepare the collection for referenced commits
	let mut referenced_commits = Vec::new();

	// Follow Git revision references
	for git_revision in &commit.referenced_commits.git_commits {
		// Lookup the reference
		if let Ok(referenced_commit) = index.lookup_git_revision(git_revision.as_str()) {
			// Ensure this is an unvisited commit
			if visited_commits.contains(referenced_commit.git_revision.as_str()) {
				continue;
			}

			// Process the referenced commit
			if let Some(referenced) =
				visit_commit(index, visited_commits, jira_tickets_set, referenced_commit)
					.with_context(|| "recursive operation failed")?
			{
				referenced_commits.push(referenced);
			}
		} else {
			eprintln!(
				"[WARNING] Git revision {git_revision} referenced by commit {} could not be found.",
				commit.git_revision
			);
		}
	}

	// Follow SVN revision references
	for svn_revision in &commit.referenced_commits.svn_commits {
		// Lookup the reference
		if let Ok(referenced_commit) = index.lookup_svn_revision(*svn_revision) {
			// Ensure this is an unvisited commit
			if visited_commits.contains(referenced_commit.git_revision.as_str()) {
				continue;
			}

			// Process the referenced commit
			if let Some(referenced) =
				visit_commit(index, visited_commits, jira_tickets_set, referenced_commit)
					.with_context(|| "recursive operation failed")?
			{
				referenced_commits.push(referenced);
			}
		} else {
			eprintln!(
				"[WARNING] SVN revision {svn_revision} referenced by commit {} could not be found.",
				commit.git_revision
			);
		}
	}

	Ok(Some(IncludedCommit {
		commit,
		referenced_commits,
	}))
}

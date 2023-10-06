//! The module for searching the repo for specific commit data.

// Uses
use std::{
	collections::HashSet,
	hash::{Hash, Hasher},
	path::Path,
	process::Command,
};

use anyhow::{Context, Result};
use shell_words::split as split_shell_words;

use crate::{
	collection::Commit,
	index::Index,
	util::{inside_out_result, run_command},
};

#[derive(Clone, Debug)]
pub struct IncludedCommit<'a> {
	pub commit:             &'a Commit,
	pub referenced_commits: Vec<IncludedCommit<'a>>,
	pub visitation_num:     usize,
}

// Since the Git revision is already a hash and will be unique, this
// implementation just forwards to it.
impl<'a> Eq for IncludedCommit<'a> {}

impl<'a> PartialEq for IncludedCommit<'a> {
	fn eq(&self, other: &Self) -> bool {
		self.commit == other.commit
	}
}

impl<'a> Hash for IncludedCommit<'a> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.commit.hash(state);
	}
}

pub fn get_search_results<'a, P>(
	index: &Index<'a>,
	repo_dir: P,
	revspec: &str,
	include_merge_commits: bool,
	affected_filepaths: &[String],
) -> Result<Vec<IncludedCommit<'a>>>
where
	P: AsRef<Path>,
{
	// Split the provided revspec into separate arguments so that Git understands
	// them (this is so that the revspec can be provided with spaces)
	let revspec_args = split_shell_words(revspec)
		.with_context(|| "unable to parse the revspec into separate arguments")?;

	// Prepare the `git log` command for the search
	let mut command = Command::new("git");
	command
		.arg("log")
		.arg("--pretty=format:%H") // Just the hashes
		.args(revspec_args.as_slice())
		.current_dir(repo_dir);
	if !include_merge_commits {
		command.arg("--no-merges");
	}
	if !affected_filepaths.is_empty() {
		command.arg("--"); // This is necessary to separate the filepaths from the revspec/commits
		command.args(affected_filepaths);
	}

	// Run the command
	let commit_list_raw = run_command(command).with_context(|| "unable to get the repo log")?;
	let commit_list = commit_list_raw
		.lines()
		.filter_map(|line| {
			let line = line.trim();
			(!line.is_empty()).then(|| {
				index
					.lookup_git_revision(line)
					.expect("all commits returned as search results should be in the index")
			})
		})
		.collect::<Vec<_>>();

	// This exists to prevent circular references and processing the same commit
	// multiple times
	let mut visited_commits = HashSet::new();

	// The need for this is a little bizarre. Basically, we want direct search
	// results to always appear on the top level (no nesting), so their Jira tickets
	// get processed etc. To accomplish this, we preliminarily block them from being
	// processed recursively.
	// The `recursion_has_happened` flag is always false for the top-level
	// processing, but in all recursive processing, it's true. When false, we ignore
	// the `visited_commits` list altogether.
	visited_commits.extend(
		commit_list
			.iter()
			.map(|commit| commit.git_revision.as_str()),
	);

	// Process each commit and build a final list of all tickets, commits, merges,
	// etc.
	let included_commits = commit_list
		.iter()
		.map(|commit| visit_commit(index, &mut visited_commits, false, commit))
		.filter_map(inside_out_result)
		.collect::<Result<Vec<_>>>()
		.with_context(|| "unable to process the commit search results")?;

	Ok(included_commits)
}

fn visit_commit<'a>(
	index: &Index<'a>,
	visited_commits: &mut HashSet<&'a str>,
	recursion_has_happened: bool,
	commit: &'a Commit,
) -> Result<Option<IncludedCommit<'a>>> {
	// Store the commit in the visited list and check to ensure that it's new
	let visited_previously = !visited_commits.insert(commit.git_revision.as_str());
	if recursion_has_happened && visited_previously {
		return Ok(None);
	}

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
			if let Some(referenced) = visit_commit(index, visited_commits, true, referenced_commit)
				.with_context(|| "recursive operation failed")?
			{
				referenced_commits.push(referenced);
			}
		} else {
			eprintln!(
				"[WARNING] Git revision `{git_revision}` referenced by commit `{}` could not be \
				 found.",
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
			if let Some(referenced) = visit_commit(index, visited_commits, true, referenced_commit)
				.with_context(|| "recursive operation failed")?
			{
				referenced_commits.push(referenced);
			}
		} else {
			eprintln!(
				"[WARNING] SVN revision `{svn_revision}` referenced by commit `{}` could not be \
				 found.",
				commit.git_revision
			);
		}
	}

	Ok(Some(IncludedCommit {
		commit,
		referenced_commits,
		visitation_num: visited_commits.len(),
	}))
}

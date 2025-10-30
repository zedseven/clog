//! The module for searching the repo for specific commit data.

// Uses
use std::{
	collections::{HashSet, VecDeque},
	hash::{Hash, Hasher},
	path::Path,
	process::Command,
};

use anyhow::{Context, Result};
use shell_words::split as split_shell_words;

use crate::{
	collection::{Commit, CommitType},
	index::Index,
	util::{inside_out_result, run_command},
};

/// A commit with its references packed alongside it, ready for display as a
/// search result.
#[derive(Clone, Debug)]
pub struct IncludedCommit<'a> {
	pub commit:         &'a Commit,
	pub linked_commits: Vec<IncludedCommit<'a>>,
}

// Since the Git revision is already a hash and will be unique, this
// implementation just forwards to it.
impl Eq for IncludedCommit<'_> {}

impl PartialEq for IncludedCommit<'_> {
	fn eq(&self, other: &Self) -> bool {
		self.commit == other.commit
	}
}

impl Hash for IncludedCommit<'_> {
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

	build_commit_inclusion_tree(index, commit_list.as_slice(), true, false)
}

pub fn get_branches_containing<P>(
	repo_dir: P,
	commit_revision: &str,
	local_branches: bool,
) -> Result<Vec<String>>
where
	P: AsRef<Path>,
{
	// Prepare the `git branch` command for the search
	let mut command = Command::new("git");
	command
		.arg("branch")
		.arg("--contains")
		.arg(commit_revision)
		.current_dir(repo_dir);
	if !local_branches {
		command.arg("--remotes");
	}

	// Run the command
	let branch_list_raw = run_command(command)
		.with_context(|| format!("unable to get the branches that contain {commit_revision}"))?;
	let branch_list = branch_list_raw
		.lines()
		.filter_map(|line| {
			let line = line.trim();
			(!line.is_empty()).then(|| line.to_owned())
		})
		.collect::<Vec<_>>();

	Ok(branch_list)
}

pub fn get_tags_containing<P>(repo_dir: P, commit_revision: &str) -> Result<Vec<String>>
where
	P: AsRef<Path>,
{
	// Prepare the `git branch` command for the search
	let mut command = Command::new("git");
	command
		.arg("tag")
		.arg("--contains")
		.arg(commit_revision)
		.current_dir(repo_dir);

	// Run the command
	let tag_list_raw = run_command(command)
		.with_context(|| format!("unable to get the tags that contain {commit_revision}"))?;
	let tag_list = tag_list_raw
		.lines()
		.filter_map(|line| {
			let line = line.trim();
			(!line.is_empty()).then(|| line.to_owned())
		})
		.collect::<Vec<_>>();

	Ok(tag_list)
}

pub fn build_commit_inclusion_tree<'a>(
	index: &Index<'a>,
	commit_list: &[&'a Commit],
	traverse_forward_references: bool,
	only_consider_likely_merges: bool,
) -> Result<Vec<IncludedCommit<'a>>> {
	// This exists to prevent circular references and processing the same commit
	// multiple times
	let mut visited_commits = HashSet::new();

	// The need for this is a little bizarre. Basically, we want direct search
	// results to always appear on the top level (no nesting), so their Jira tickets
	// get processed, etc. To accomplish this, we preliminarily block them from
	// being processed recursively.
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
		.map(|commit| {
			visit_commit(
				index,
				&mut visited_commits,
				traverse_forward_references,
				only_consider_likely_merges,
				false,
				commit,
			)
		})
		.filter_map(inside_out_result)
		.collect::<Result<Vec<_>>>()
		.with_context(|| "unable to process the commit search results")?;

	Ok(included_commits)
}

pub fn flatten_inclusion_tree<'a>(inclusion_tree: &[IncludedCommit<'a>]) -> Vec<&'a Commit> {
	let mut flattened_commit_list = Vec::new();
	let mut commits_to_visit = VecDeque::new();
	commits_to_visit.extend(inclusion_tree);

	while let Some(included_commit) = commits_to_visit.pop_front() {
		flattened_commit_list.push(included_commit.commit);
		commits_to_visit.extend(&included_commit.linked_commits);
	}

	flattened_commit_list
}

fn visit_commit<'a>(
	index: &Index<'a>,
	visited_commits: &mut HashSet<&'a str>,
	traverse_forward_references: bool,
	only_consider_likely_merges: bool,
	recursion_has_happened: bool,
	commit: &'a Commit,
) -> Result<Option<IncludedCommit<'a>>> {
	// Store the commit in the visited list and check to ensure that it's new
	let visited_previously = !visited_commits.insert(commit.git_revision.as_str());
	if recursion_has_happened && visited_previously {
		return Ok(None);
	}

	// Process all forward references of the commit
	let raw_references = if traverse_forward_references {
		index.get_commit_forward_references(commit)
	} else {
		index.get_commit_backward_references(commit)
	};
	let linked_commits = raw_references
		.iter()
		.filter(|referenced_commit| {
			!only_consider_likely_merges
				|| referenced_commit.likely_commit_type == CommitType::CherryPick
		})
		.map(|referenced_commit| {
			visit_commit(
				index,
				visited_commits,
				traverse_forward_references,
				only_consider_likely_merges,
				true,
				referenced_commit,
			)
		})
		.filter_map(inside_out_result)
		.collect::<Result<Vec<_>>>()
		.with_context(|| "recursive operation failed")?;

	Ok(Some(IncludedCommit {
		commit,
		linked_commits,
	}))
}

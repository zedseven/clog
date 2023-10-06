//! The module for commit data collection.

// Uses
use std::{
	collections::HashSet,
	hash::{Hash, Hasher},
	path::Path,
	process::Command,
};

use anyhow::{anyhow, Context, Result};
use lazy_static::lazy_static;
use linked_hash_set::LinkedHashSet;
use regex::Regex;

use crate::{
	constants::{GIT_SVN_ID_STR, SHA1_HASH_ASCII_LENGTH},
	util::run_command,
};

// Constants
const LOG_COMMIT_DELIMITER: &str = "CLOG-COMMIT-DELIMITER\n";

#[derive(Debug)]
pub struct Commit {
	pub git_revision:       String,
	pub svn_info:           Option<SvnInfo>,
	pub jira_tickets:       Vec<String>,
	pub referenced_commits: ReferencedCommits,
}

#[derive(Debug)]
pub struct SvnInfo {
	pub svn_url:      String,
	pub svn_revision: u32,
}

#[derive(Debug)]
pub struct ReferencedCommits {
	pub git_commits: Vec<String>,
	pub svn_commits: Vec<u32>,
}

// Since the Git revision is already a hash and will be unique, this
// implementation just forwards to it.
impl Eq for Commit {}

impl PartialEq for Commit {
	fn eq(&self, other: &Self) -> bool {
		self.git_revision == other.git_revision
	}
}

impl Hash for Commit {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.git_revision.hash(state);
	}
}

pub fn get_complete_commit_list<P>(
	repo_dir: P,
	include_mentioned_jira_tickets: bool,
) -> Result<Vec<Commit>>
where
	P: AsRef<Path>,
{
	// Prepare the `git log` command for collecting all commits in the repo
	let mut command = Command::new("git");
	command
		.arg("log")
		.arg("--all")
		.arg("--full-history")
		.arg(format!("--pretty=format:{LOG_COMMIT_DELIMITER}%H\n%s\n%b"))
		.current_dir(repo_dir);

	// Run the command
	run_command(command)
		.with_context(|| "unable to get the repo log")?
		// Split the output by the delimiter to get one entry per commit
		.split(LOG_COMMIT_DELIMITER)
		// Since it's a split() operation, the first delimiter at the beginning leads to an empty
		// entry at the top
		.skip(1)
		// Process each entry into a usable commit
		.map(|entry| process_commit_entry(entry, include_mentioned_jira_tickets))
		.collect::<Result<Vec<_>>>()
		.with_context(|| "unable to process log entries")
}

fn process_commit_entry(entry: &str, include_mentioned_jira_tickets: bool) -> Result<Commit> {
	let lines = entry.lines().collect::<Vec<_>>();
	if lines.is_empty() {
		return Err(anyhow!(
			"commit entry is missing the commit hash (impossible)"
		));
	}

	let git_revision_str = lines[0];
	if git_revision_str.len() != SHA1_HASH_ASCII_LENGTH {
		return Err(anyhow!("SHA-1 hash is of invalid length"));
	}

	let git_revision = git_revision_str.to_owned();

	// Search the commit message content for information
	let mut svn_info = None;
	let mut jira_tickets_set = HashSet::new();
	let mut referenced_git_commits_set = LinkedHashSet::new();
	let mut referenced_svn_commits_set = LinkedHashSet::new();
	let mut first_line = true;
	for line in lines.iter().skip(1) {
		// Search for the SVN metadata string
		if svn_info.is_none() && line.starts_with(GIT_SVN_ID_STR) {
			// The SVN metadata looks like this (without quotes):
			// `git-svn-id: <URL>@<REVISION> <UUID>`
			let line_parts = line.trim().split(' ').collect::<Vec<_>>();
			if line_parts.len() != 3 {
				return Err(anyhow!("{GIT_SVN_ID_STR} line is invalid"));
			}
			let svn_info_str = line_parts[1];
			let (svn_url_str, svn_revision_str) = svn_info_str
				.split_once('@')
				.ok_or_else(|| anyhow!("SVN info is invalid"))?;

			let svn_url = svn_url_str.to_owned();
			let svn_revision = str::parse(svn_revision_str)
				.with_context(|| "unable to parse SVN revision number as an integer")?;

			svn_info = Some(SvnInfo {
				svn_url,
				svn_revision,
			});

			// If we don't continue here, the UUID in the SVN metadata may be mistaken for a
			// Git hash
			continue;
		}

		// Search for Jira tickets
		lazy_static! {
			/// Looks for a Jira ticket right at the start, skipping "Pull request #..."
			static ref JIRA_TICKET_START_REGEX: Regex =
				Regex::new(r"^\s*(?:Pull request #\d+.*?)?([A-Z][A-Z0-9_]+-[1-9][0-9]*)\b").unwrap();
			/// Looks for a Jira ticket anywhere on the line
			static ref JIRA_TICKET_REFERENCED_REGEX: Regex =
				Regex::new(r"\b([A-Z][A-Z0-9_]+-[1-9][0-9]*)\b").unwrap();
			/// Matches any Git commit hashes 7 characters or longer (to avoid matching small numbers that show up for other reasons)
			static ref GIT_COMMIT_REFERENCE_REGEX: Regex =
				Regex::new(r"(?i)\b([0-9a-f]{7,40})\b").unwrap();
			/// Finds (hopefully) all references to SVN revisions, but returns them as a group, not individually
			static ref SVN_COMMIT_REFERENCE_REGEX: Regex =
				Regex::new(r"(?i)\b(?:(?:commit|revision|rev)(?:s|\(s\))? |r)(\d+(?:-\d+)?(?:, ?\d+(?:-\d+)?)*)\b").unwrap();
		}
		let jira_ticket_regex = if include_mentioned_jira_tickets {
			&*JIRA_TICKET_REFERENCED_REGEX
		} else {
			&*JIRA_TICKET_START_REGEX
		};
		// Only check the first line for Jira tickets, unless we're supposed to look for
		// all mentioned tickets
		if include_mentioned_jira_tickets || first_line {
			for jira_ticket in jira_ticket_regex.captures_iter(line) {
				jira_tickets_set.insert(jira_ticket[1].to_owned());
			}
		}

		// Search for referenced commits (merges, etc.)
		for git_commit_reference in GIT_COMMIT_REFERENCE_REGEX.captures_iter(line) {
			referenced_git_commits_set.insert(git_commit_reference[1].to_owned());
		}
		for svn_commit_reference_group in SVN_COMMIT_REFERENCE_REGEX.captures_iter(line) {
			// The result of the Regex will be a comma-delimited list of continuous
			// selections
			// Overall match: `16732, 16734-16735, 16737-16740, 16768`
			for continuous_selection in svn_commit_reference_group[1].split(',') {
				// Continuous match: `16734-16735`
				let continuous_selection = continuous_selection.trim();
				if let Some((start, end)) = continuous_selection.split_once('-') {
					// Insert all commits in the range
					let start_revision =
						str::parse::<u32>(start).expect("the string is guaranteed to be numeric");
					let end_revision =
						str::parse::<u32>(end).expect("the string is guaranteed to be numeric");
					referenced_svn_commits_set.extend(start_revision..=end_revision);
				} else {
					// Insert the one commit
					let revision = str::parse::<u32>(continuous_selection)
						.expect("the string is guaranteed to be numeric");
					referenced_svn_commits_set.insert(revision);
				}
			}
		}

		first_line = false;
	}

	Ok(Commit {
		git_revision,
		svn_info,
		jira_tickets: Vec::from_iter(jira_tickets_set),
		referenced_commits: ReferencedCommits {
			git_commits: Vec::from_iter(referenced_git_commits_set),
			svn_commits: Vec::from_iter(referenced_svn_commits_set),
		},
	})
}

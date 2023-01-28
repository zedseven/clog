// Uses
use std::{path::Path, process::Command, str::from_utf8 as str_from_utf8};

use anyhow::{anyhow, Context, Result};
use lazy_static::lazy_static;
use linked_hash_set::LinkedHashSet;
use regex::Regex;

// Constants
const SHA1_HASH_LENGTH: usize = 20;
const SHA1_HASH_ASCII_LENGTH: usize = SHA1_HASH_LENGTH * 2;
const GIT_SVN_ID_STR: &str = "git-svn-id";
const LOG_COMMIT_DELIMITER: &str = "CLOG-COMMIT-DELIMITER\n";
const MAX_ALLOWED_REFERENCED_COMMITS: usize = 20; // To help prevent errors where 500 commits get pulled in

// Types and Structures
pub type Sha1Hash = [u8; SHA1_HASH_LENGTH];
pub type GitRevision = String; // It has to stay in String format so that it remains searchable
pub type GitRevisionPartial = String;
pub type SvnRevision = u32;
pub type JiraTicket = String;

#[derive(Debug)]
pub struct Commit {
	pub git_revision: GitRevision,
	pub svn_info: Option<SvnInfo>,
	pub jira_tickets: Vec<JiraTicket>,
	pub referenced_commits: ReferencedCommits,
}

#[derive(Debug)]
pub struct SvnInfo {
	pub svn_url: String,
	pub svn_revision: SvnRevision,
}

#[derive(Debug)]
pub struct ReferencedCommits {
	pub git_commits: Vec<GitRevisionPartial>,
	pub svn_commits: Vec<SvnRevision>,
}

pub fn get_repo_revision_maps<P>(repo_dir: P) -> Result<Vec<Commit>>
where
	P: AsRef<Path>,
{
	process_log_data(
		get_complete_log(repo_dir)
			.with_context(|| "unable to get the git repo log")?
			.as_str(),
	)
	.with_context(|| "log data is invalid")
}

fn get_complete_log<P>(repo_dir: P) -> Result<String>
where
	P: AsRef<Path>,
{
	let command_result = Command::new("git")
		.arg("log")
		.arg("--all")
		.arg("--full-history")
		.arg(format!("--pretty=format:{LOG_COMMIT_DELIMITER}%H\n%s\n%b"))
		.current_dir(repo_dir)
		.output()
		.with_context(|| "unable to run git to get the repo log")?;
	if !command_result.status.success() {
		return Err(anyhow!(
			"Git command failed: {:?}",
			command_result.status.code()
		));
	}

	str_from_utf8(&command_result.stdout)
		.with_context(|| "unable to parse git command output as UTF-8")
		.map(ToOwned::to_owned)
}

fn get_revspec_log<P>(repo_dir: P, revspec: &str) -> Result<String>
where
	P: AsRef<Path>,
{
	let command_result = Command::new("git")
		.arg("log")
		.arg("--pretty=format:%H") // Just the hashes
		.arg(revspec)
		.current_dir(repo_dir)
		.output()
		.with_context(|| "unable to run git to get the repo log")?;
	if !command_result.status.success() {
		return Err(anyhow!(
			"Git command failed: {:?}",
			command_result.status.code()
		));
	}

	str_from_utf8(&command_result.stdout)
		.with_context(|| "unable to parse git command output as UTF-8")
		.map(ToOwned::to_owned)
}

fn process_log_data(git_log: &str) -> Result<Vec<Commit>> {
	let result = git_log
		.split(LOG_COMMIT_DELIMITER)
		.skip(1)
		.map(process_commit_entry)
		.collect::<Result<Vec<_>>>()
		.with_context(|| "unable to process log entries")?;

	// Sort the results
	// result.sort_by_key(|mapping| mapping.svn_revision);

	dbg!(&result);

	Ok(result)
}

fn process_commit_entry(entry: &str) -> Result<Commit> {
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
	// let git_revision =
	// 	vec_to_arr(parse_hex_str(git_revision_str).with_context(|| "unable to parse
	// SHA-1 hash")?);

	// Search the commit message content for information
	let mut svn_info = None;
	let mut jira_tickets_set = LinkedHashSet::new();
	let mut referenced_git_commits_set = LinkedHashSet::new();
	let mut referenced_svn_commits_set = LinkedHashSet::new();
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
			static ref JIRA_TICKET_REGEX: Regex =
				Regex::new(r"\b([A-Z][A-Z0-9_]+-[1-9][0-9]*)\b").unwrap();
			/// Matches any Git commit hashes 7 characters or longer (to avoid matching small numbers that show up for other reasons)
			static ref GIT_COMMIT_REFERENCE_REGEX: Regex =
				Regex::new(r"(?i)\b([0-9a-f]{7,40})\b").unwrap();
			/// Finds (hopefully) all references to SVN revisions, but returns them as a group, not individually
			static ref SVN_COMMIT_REFERENCE_REGEX: Regex =
				Regex::new(r"(?i)\b(?:(?:commit|revision|rev)(?:s|\(s\))? |r)(\d+(?:-\d+)?(?:, ?\d+(?:-\d+)?)*)\b").unwrap();
		}
		for jira_ticket in JIRA_TICKET_REGEX.captures_iter(line) {
			jira_tickets_set.insert_if_absent(jira_ticket[1].to_owned());
		}

		// Search for referenced commits (merges, etc.)
		for git_commit_reference in GIT_COMMIT_REFERENCE_REGEX.captures_iter(line) {
			referenced_git_commits_set.insert_if_absent(git_commit_reference[1].to_owned());
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
					let start_revision = str::parse::<SvnRevision>(start)
						.expect("the string is guaranteed to be numeric");
					let end_revision = str::parse::<SvnRevision>(end)
						.expect("the string is guaranteed to be numeric");
					referenced_svn_commits_set.extend(start_revision..=end_revision);
				} else {
					// Insert the one commit
					let revision = str::parse::<SvnRevision>(continuous_selection)
						.expect("the string is guaranteed to be numeric");
					referenced_svn_commits_set.insert_if_absent(revision);
				}
			}
		}
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

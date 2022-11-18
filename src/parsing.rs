// Uses
use std::{path::Path, process::Command, str::from_utf8 as str_from_utf8};

use anyhow::{anyhow, Context, Result};

use crate::util::{parse_hex_str_strict, vec_to_arr};

// Constants
const SHA1_HASH_LENGTH: usize = 20;
const SHA1_HASH_ASCII_LENGTH: usize = SHA1_HASH_LENGTH * 2;
const GIT_SVN_ID_STR: &str = "git-svn-id";
const LOG_IDENTIFIER: &str = "BUILD-REVMAP-COMMIT\n";

pub type Sha1Hash = [u8; SHA1_HASH_LENGTH];

#[derive(Debug)]
pub struct RevisionMapping {
	pub svn_url: String,
	pub svn_revision: u32,
	pub git_revision: Sha1Hash,
}

pub fn get_repo_revision_maps<P>(repo_dir: P) -> Result<Vec<RevisionMapping>>
where
	P: AsRef<Path>,
{
	process_log_data(
		get_log(repo_dir)
			.with_context(|| "unable to get the git repo log")?
			.as_str(),
	)
	.with_context(|| "log data is invalid")
}

fn get_log<P>(repo_dir: P) -> Result<String>
where
	P: AsRef<Path>,
{
	let command_result = Command::new("git")
		.arg("log")
		.arg("--all")
		.arg("--full-history")
		.arg(format!("--pretty=format:{LOG_IDENTIFIER}%H\n%s\n%b"))
		.current_dir(repo_dir)
		.output()
		.with_context(|| "unable to run git to get the repo log")?;
	if !command_result.status.success() {
		return Err(anyhow!(
			"git command failed: {:?}",
			command_result.status.code()
		));
	}

	str_from_utf8(&command_result.stdout)
		.with_context(|| "unable to parse git command output as UTF-8")
		.map(ToOwned::to_owned)
}

fn process_log_data(git_log: &str) -> Result<Vec<RevisionMapping>> {
	let mut result = git_log
		.split(LOG_IDENTIFIER)
		.skip(1)
		.map(process_commit_entry)
		.collect::<Result<Vec<_>>>()
		.with_context(|| "unable to process log entries")?;

	// Sort the results
	result.sort_by_key(|mapping| mapping.svn_revision);

	Ok(result)
}

fn process_commit_entry(entry: &str) -> Result<RevisionMapping> {
	let mut result = None;

	let lines = entry.lines().collect::<Vec<_>>();
	if lines.len() <= 1 {
		return Err(anyhow!("commit entry has too few lines"));
	}

	let git_revision_str = lines[0];
	if git_revision_str.len() != SHA1_HASH_ASCII_LENGTH {
		return Err(anyhow!("SHA1 hash is of invalid length"));
	}

	let git_revision = vec_to_arr(
		parse_hex_str_strict(git_revision_str).with_context(|| "unable to parse SHA1 hash")?,
	);

	for line in lines.iter().skip(1) {
		if !line.starts_with(GIT_SVN_ID_STR) {
			continue;
		}

		let line_parts = line.split(' ').collect::<Vec<_>>();
		if line_parts.len() != 3 {
			return Err(anyhow!("{GIT_SVN_ID_STR} line is invalid"));
		}
		let svn_info = line_parts[1];
		let (svn_url_str, svn_revision_str) = svn_info
			.split_once('@')
			.ok_or_else(|| anyhow!("SVN info is invalid"))?;

		let svn_url = svn_url_str.to_owned();
		let svn_revision = str::parse(svn_revision_str)
			.with_context(|| "unable to parse SVN revision number as an integer")?;

		result = Some(RevisionMapping {
			svn_url,
			svn_revision,
			git_revision,
		});

		break;
	}

	result.ok_or_else(|| anyhow!("unable to find a {GIT_SVN_ID_STR} line"))
}

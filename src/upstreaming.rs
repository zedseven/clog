// Uses
use std::{
	collections::{HashMap, HashSet},
	path::Path,
	process::Command,
	sync::LazyLock,
};

use anyhow::{Context, Result};
use regex::Regex;

use crate::util::{run_command, run_command_for_exit_code};

// Constants
/// https://stackoverflow.com/questions/171550/find-out-which-remote-branch-a-local-branch-is-tracking/9753364#9753364
const UPSTREAM_SUFFIX: &str = "@{u}";

// Types
pub type RemoteBranchDatabase = HashMap<String, HashSet<String>>;

pub fn upstream_revspec<P>(
	repo_dir: P,
	remote_branch_database: &RemoteBranchDatabase,
	revspec: &str,
) -> Result<String>
where
	P: AsRef<Path>,
{
	/// Used for splitting the revspec into individual refs.
	static REVSPEC_REF_SPLITTING_REGEX: LazyLock<Regex> =
		LazyLock::new(|| Regex::new(r"\s+|\.{2,3}|@\{.*\}|\^(?:-\d+|[!@])?|[~?\[]").unwrap());

	// Split the revspec into refs, and for each ref, add the upstream suffix if it
	// has an upstream
	let mut revspec_result = String::with_capacity(revspec.len() * 2);
	let mut last_index = 0;
	for non_ref_match in REVSPEC_REF_SPLITTING_REGEX.find_iter(revspec) {
		let reference = &revspec[last_index..non_ref_match.start()];
		let non_ref_text = &revspec[non_ref_match.start()..non_ref_match.end()];

		last_index = non_ref_match.end();

		let should_upstream_ref = !reference.is_empty() && non_ref_text != UPSTREAM_SUFFIX;

		if should_upstream_ref {
			revspec_result.push_str(
				upstream_ref_if_possible(&repo_dir, remote_branch_database, reference)?.as_str(),
			);
			revspec_result.push_str(non_ref_text);
		} else {
			revspec_result.push_str(reference);
			revspec_result.push_str(non_ref_text);
		}
	}

	{
		let reference = &revspec[last_index..];

		if !reference.is_empty() {
			revspec_result.push_str(
				upstream_ref_if_possible(&repo_dir, remote_branch_database, reference)?.as_str(),
			);
		}
	}

	Ok(revspec_result)
}

pub fn upstream_ref_if_possible<P>(
	repo_dir: P,
	remote_branch_database: &RemoteBranchDatabase,
	reference: &str,
) -> Result<String>
where
	P: AsRef<Path>,
{
	let reference_with_upstream = format!("{reference}{UPSTREAM_SUFFIX}");

	// Prepare the `git log` command for the search
	let mut command = Command::new("git");
	command
		.arg("rev-parse")
		.arg(reference_with_upstream.as_str())
		.current_dir(repo_dir);

	// Run the command to check whether it's got an upstream
	let is_local_branch_with_upstream = run_command_for_exit_code(command)?;

	if is_local_branch_with_upstream {
		return Ok(reference_with_upstream);
	}

	// Check if a remote branch exists with the same name
	for (remote, branch_set) in remote_branch_database {
		if branch_set.contains(reference) {
			return Ok(format!("{remote}/{reference}"));
		}
	}

	// Return the raw reference since nothing was found
	Ok(reference.to_owned())
}

pub fn build_remote_branch_database<P>(repo_dir: P) -> Result<RemoteBranchDatabase>
where
	P: AsRef<Path>,
{
	const HEAD_MARKER_ARROW: &str = "HEAD -> ";

	// Prepare the command to get the list of remotes
	let mut command = Command::new("git");
	command.arg("remote").current_dir(&repo_dir);

	let mut remotes = run_command(command)
		.with_context(|| "unable to get the list of remotes")?
		.lines()
		.map(ToOwned::to_owned)
		.collect::<Vec<_>>();

	remotes.sort_unstable_by_key(String::len);
	remotes.reverse();

	// Prepare the command to get the remote branches
	let mut command = Command::new("git");
	command
		.arg("branch")
		.arg("--list")
		.arg("--remotes")
		.current_dir(repo_dir);

	// Build the "database" of remote branches
	let mut remote_branch_database: RemoteBranchDatabase = HashMap::new();
	for line in run_command(command)
		.with_context(|| "unable to get the list of remote branches")?
		.lines()
	{
		let line_trimmed = line.trim();

		if line_trimmed.contains(HEAD_MARKER_ARROW) {
			continue;
		}

		for remote in &remotes {
			if line_trimmed.starts_with(remote) {
				let remote_branch = &line_trimmed[(remote.len() + 1)..].to_owned();
				remote_branch_database
					.entry(remote.clone())
					.and_modify(|branch_set| {
						branch_set.insert(remote_branch.clone());
					})
					.or_insert_with(|| {
						let mut branch_set = HashSet::new();
						branch_set.insert(remote_branch.clone());
						branch_set
					});
			}
		}
	}

	Ok(remote_branch_database)
}

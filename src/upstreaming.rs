// Uses
use std::{path::Path, process::Command, sync::LazyLock};

use anyhow::Result;
use regex::Regex;

use crate::util::run_command_for_exit_code;

// Constants
/// https://stackoverflow.com/questions/171550/find-out-which-remote-branch-a-local-branch-is-tracking/9753364#9753364
const UPSTREAM_SUFFIX: &str = "@{u}";

pub fn upstream_revspec<P>(repo_dir: P, revspec: &str) -> Result<String>
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
			revspec_result.push_str(upstream_ref_if_possible(&repo_dir, reference)?.as_str());
			revspec_result.push_str(non_ref_text);
		} else {
			revspec_result.push_str(reference);
			revspec_result.push_str(non_ref_text);
		}
	}

	{
		let reference = &revspec[last_index..];

		if !reference.is_empty() {
			revspec_result.push_str(upstream_ref_if_possible(&repo_dir, reference)?.as_str());
		}
	}

	Ok(revspec_result)
}

pub fn upstream_ref_if_possible<P>(repo_dir: P, reference: &str) -> Result<String>
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

	// Run the command
	let is_branch_with_upstream = run_command_for_exit_code(command)?;

	Ok(if is_branch_with_upstream {
		reference_with_upstream
	} else {
		reference.to_owned()
	})
}

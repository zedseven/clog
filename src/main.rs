//! A simple utility for building a revision map for a Git repo that was
//! converted from an SVN repository by `git-svn` with the `--metadata` flag
//! supplied.

// Linting Rules
#![warn(
	clippy::complexity,
	clippy::correctness,
	clippy::pedantic,
	clippy::perf,
	clippy::style,
	clippy::suspicious,
	clippy::clone_on_ref_ptr,
	clippy::dbg_macro,
	clippy::decimal_literal_representation,
	clippy::exit,
	clippy::filetype_is_file,
	clippy::if_then_some_else_none,
	clippy::non_ascii_literal,
	clippy::self_named_module_files,
	clippy::str_to_string,
	clippy::undocumented_unsafe_blocks,
	clippy::wildcard_enum_match_arm
)]
#![allow(
	clippy::cast_possible_truncation,
	clippy::cast_possible_wrap,
	clippy::cast_precision_loss,
	clippy::cast_sign_loss,
	clippy::doc_markdown,
	clippy::module_name_repetitions,
	clippy::similar_names,
	clippy::too_many_lines,
	clippy::unnecessary_wraps,
	dead_code,
	unused_macros
)]

// Modules
mod cli;
mod collection;
mod constants;
mod index;
mod search;
mod util;

// Uses
use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result};

use crate::{
	cli::build_cli,
	collection::{get_complete_commit_list, Commit},
	index::Index,
	search::{get_search_results, IncludedCommit},
};

// Entry Point
fn main() -> Result<()> {
	let cli_definition = build_cli();
	let subcommand_matches = cli_definition.get_matches();

	match subcommand_matches.subcommand() {
		Some(("list", matches)) => {
			// Get the CLI arguments that were provided
			let repo_dir = matches
				.get_one::<String>("repo")
				.expect("Clap ensures the argument is provided");
			let revspec = matches
				.get_one::<String>("revspec")
				.expect("Clap ensures the argument is provided");
			let affected_filepaths = matches.get_one::<String>("filepaths");
			let include_referenced_jira_tickets = *matches
				.get_one::<bool>("referenced-tickets")
				.unwrap_or(&false);
			let flatten = *matches.get_one::<bool>("flatten").unwrap_or(&false);
			let hash_length = *matches
				.get_one::<u32>("hash-length")
				.expect("Clap provides a default value") as usize;

			let commits =
				get_complete_commit_list(repo_dir.as_str(), include_referenced_jira_tickets)
					.with_context(|| "unable to get the repo revision maps")?;

			let index = Index::new(commits.as_slice())?;

			let search_results = get_search_results(
				&index,
				repo_dir.as_str(),
				revspec.as_str(),
				affected_filepaths.map(String::as_str),
			)
			.with_context(|| "unable to perform the search")?;

			if flatten {
				let mut commit_list = HashSet::new();
				let mut jira_ticket_list = HashSet::new();

				for included_commit in &search_results {
					flatten_search_results(
						&mut commit_list,
						&mut jira_ticket_list,
						included_commit,
					);
				}

				println!("Commits: ({} total)", commit_list.len());
				for commit in commit_list {
					println!("- {}", &commit.git_revision[0..hash_length]);
				}

				println!();

				println!("Jira tickets: ({} total)", jira_ticket_list.len());
				for jira_ticket in jira_ticket_list {
					println!("- {jira_ticket}");
				}
			} else {
				let mut jira_ticket_groups = HashMap::new();

				for included_commit in &search_results {
					group_results_by_jira_ticket(&mut jira_ticket_groups, included_commit);
				}

				let mut jira_ticket_groups_sorted = jira_ticket_groups.iter().collect::<Vec<_>>();
				jira_ticket_groups_sorted.sort_by_key(|entry| {
					let (project, issue) = entry.0.split_once('-').expect(
						"all Jira tickets should have 1 hyphen separating the project from the \
						 issue number",
					);
					let issue_num = issue
						.parse::<u32>()
						.expect("all issue numbers should be numeric");
					(project, issue_num)
				});

				println!("Jira tickets: ({} total)", jira_ticket_groups_sorted.len());
				for (jira_ticket, commits) in jira_ticket_groups_sorted {
					println!("- {jira_ticket}:");
					for commit in commits {
						println!("\t- {}", &commit.git_revision[0..hash_length]);
					}
				}
			}
		}
		Some(("revmap", matches)) => {
			let repo_dir = matches
				.get_one::<String>("repo")
				.expect("Clap ensures the argument is provided");

			// if let Some(path) = matches.get_one::<String>("binary") {
			// 	write_to_bin(path, revision_maps.as_slice())
			// 		.with_context(|| "unable to write the revision map to binary")?;
			// } else if let Some(path) = matches.get_one::<String>("markdown")
			// { 	let git_url_base = matches
			// 		.get_one::<String>("git-url-base")
			// 		.expect("Clap ensures the argument is provided");
			// 	write_to_markdown(path, git_url_base.as_str(),
			// revision_maps.as_slice()) 		.with_context(|| "unable to write the
			// revision map to markdown")?; } else if let Some(path) =
			// matches.get_one::<String>("markdown-basic") {
			// 	write_to_markdown_basic(path, revision_maps.as_slice())
			// 		.with_context(|| "unable to write the revision map to
			// markdown")?; } else {
			// 	unreachable!("Clap ensures exactly one output path is provided");
			// };
		}
		_ => unreachable!("Clap ensures that a subcommand is provided"),
	}

	Ok(())
}

fn group_results_by_jira_ticket<'a>(
	jira_ticket_groups: &mut HashMap<&'a str, Vec<&'a Commit>>,
	included_commit: &'a IncludedCommit,
) {
	// Build the grouping
	for jira_ticket in &included_commit.commit.jira_tickets {
		jira_ticket_groups
			.entry(jira_ticket.as_str())
			.and_modify(|commits| commits.push(included_commit.commit))
			.or_insert(vec![included_commit.commit]);
	}

	// Recurse over the referenced commits
	for referenced_commit in &included_commit.referenced_commits {
		group_results_by_jira_ticket(jira_ticket_groups, referenced_commit);
	}
}

fn flatten_search_results<'a>(
	commit_list: &mut HashSet<&'a Commit>,
	jira_ticket_list: &mut HashSet<&'a str>,
	included_commit: &'a IncludedCommit,
) {
	// Add the commit to the list
	commit_list.insert(included_commit.commit);

	// Collect the Jira tickets
	jira_ticket_list.extend(
		included_commit
			.commit
			.jira_tickets
			.iter()
			.map(String::as_str),
	);

	// Recurse over the referenced commits
	for referenced_commit in &included_commit.referenced_commits {
		flatten_search_results(commit_list, jira_ticket_list, referenced_commit);
	}
}

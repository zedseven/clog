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
mod writing;

// Uses
use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result};
use shell_words::split as split_shell_words;

use crate::{
	cli::build_cli,
	collection::{get_complete_commit_list, Commit},
	index::Index,
	search::{get_search_results, IncludedCommit},
	util::sortable_jira_ticket,
	writing::{write_to_bin, write_to_markdown},
};

// Entry Point
fn main() -> Result<()> {
	let cli_definition = build_cli();
	let subcommand_matches = cli_definition.get_matches();

	match subcommand_matches.subcommand() {
		Some(("list", matches)) => {
			// Collect the CLI arguments that were provided
			let repo_dir = matches
				.get_one::<String>("repo")
				.expect("Clap ensures the argument is provided");
			let revspec = matches
				.get_one::<String>("revspec")
				.expect("Clap ensures the argument is provided");
			let affected_filepath_sets = matches.get_many::<String>("filepath");
			let include_merge_commits =
				*matches.get_one::<bool>("include-merges").unwrap_or(&false);
			let include_mentioned_jira_tickets = *matches
				.get_one::<bool>("include-mentioned")
				.unwrap_or(&false);
			let flatten = *matches.get_one::<bool>("flatten").unwrap_or(&false);
			let hash_length = *matches
				.get_one::<u32>("hash-length")
				.expect("Clap provides a default value") as usize;

			// Since the filepaths can be provided all in one argument, or separately with
			// multiple arguments, they need to be collected into a single list
			let mut affected_filepaths = Vec::new();
			if let Some(filepath_sets) = affected_filepath_sets {
				for filepath_set in filepath_sets {
					affected_filepaths.extend(
						split_shell_words(filepath_set.as_str()).with_context(|| {
							format!("unable to parse filepath set: {filepath_set}")
						})?,
					);
				}
			}

			// Collect all commits in the repo
			let commits =
				get_complete_commit_list(repo_dir.as_str(), include_mentioned_jira_tickets)
					.with_context(|| "unable to get the repo revision maps")?;

			// Build the index
			let index = Index::new(commits.as_slice())?;

			// Perform the search
			let search_results = get_search_results(
				&index,
				repo_dir.as_str(),
				revspec.as_str(),
				include_merge_commits,
				affected_filepaths.as_slice(),
			)
			.with_context(|| "unable to perform the search")?;

			// Display the results
			if flatten {
				// Flatten the results
				let mut commit_set = HashSet::new();
				let mut jira_ticket_set = HashSet::new();

				for included_commit in &search_results {
					flatten_search_results(&mut commit_set, &mut jira_ticket_set, included_commit);
				}

				// Sort the lists
				let mut commit_list = Vec::from_iter(commit_set);
				let mut jira_ticket_list = Vec::from_iter(jira_ticket_set);

				commit_list.sort_unstable_by_key(|commit| commit.visitation_num);
				jira_ticket_list
					.sort_unstable_by_key(|jira_ticket| sortable_jira_ticket(jira_ticket));

				// Display the results
				println!("Commits: ({} total)", commit_list.len());
				for commit in commit_list {
					println!("- {}", &commit.commit.git_revision[0..hash_length]);
				}

				println!();

				if include_mentioned_jira_tickets {
					println!(
						"Jira tickets: ({} total, including referenced commits' tickets and \
						 tickets mentioned anywhere in commit messages)",
						jira_ticket_list.len()
					);
				} else {
					println!(
						"Jira tickets: ({} total, including referenced commits' tickets)",
						jira_ticket_list.len()
					);
				}
				for jira_ticket in jira_ticket_list {
					println!("- {jira_ticket}");
				}
			} else {
				// Group the commits by Jira ticket
				let mut jira_ticket_groups = HashMap::new();
				for included_commit in search_results {
					for jira_ticket in &included_commit.commit.jira_tickets {
						jira_ticket_groups
							.entry(jira_ticket.as_str())
							.and_modify(|ticket_commits: &mut Vec<IncludedCommit>| {
								ticket_commits.push(included_commit.clone());
							})
							.or_insert(vec![included_commit.clone()]);
					}
				}

				// Sort the Jira tickets
				let mut jira_ticket_groups_sorted = jira_ticket_groups.iter().collect::<Vec<_>>();
				jira_ticket_groups_sorted
					.sort_unstable_by_key(|entry| sortable_jira_ticket(entry.0));

				// Display the results
				println!("Jira tickets: ({} total)", jira_ticket_groups_sorted.len());
				for (jira_ticket, commits) in jira_ticket_groups_sorted {
					println!("- {jira_ticket}:");
					display_commit_reference_tree(commits.as_slice(), 1, hash_length);
				}
			}
		}
		Some(("revmap", matches)) => {
			// Collect the CLI arguments that were provided
			let repo_dir = matches
				.get_one::<String>("repo")
				.expect("Clap ensures the argument is provided");
			let hash_length = *matches
				.get_one::<u32>("hash-length")
				.expect("Clap provides a default value") as usize;

			// Collect all commits in the repo
			let commits = get_complete_commit_list(repo_dir.as_str(), false)
				.with_context(|| "unable to get the repo revision maps")?;

			// Build a revision map and discard any commits that don't have SVN info
			let mut revision_map = commits
				.iter()
				.filter_map(|commit| {
					commit.svn_info.as_ref().map(|svn_info| {
						(
							svn_info.svn_revision,
							svn_info.svn_url.as_str(),
							commit.git_revision.as_str(),
						)
					})
				})
				.collect::<Vec<_>>();

			// Sort the revision map to ensure that it's in order
			revision_map.sort_by_key(|entry| entry.0); // Stable sort to preserve order in case of ties

			// Write it to disk in the specified formats
			if let Some(path) = matches.get_one::<String>("binary") {
				write_to_bin(path, revision_map.as_slice())
					.with_context(|| "unable to write the revision map to binary")?;
			}
			if let Some(path) = matches.get_one::<String>("markdown") {
				write_to_markdown(path, revision_map.as_slice(), hash_length)
					.with_context(|| "unable to write the revision map to markdown")?;
			};
		}
		_ => unreachable!("Clap ensures that a subcommand is provided"),
	}

	Ok(())
}

fn display_commit_reference_tree(
	included_commits: &[IncludedCommit],
	indentation: u32,
	hash_length: usize,
) {
	for included_commit in included_commits {
		// Print the indentation
		for _ in 0..indentation {
			print!("\t");
		}

		// Print the commit revision
		println!("- {}", &included_commit.commit.git_revision[0..hash_length]);

		// Recurse over the referenced commits
		display_commit_reference_tree(
			included_commit.referenced_commits.as_slice(),
			indentation + 1,
			hash_length,
		);
	}
}

fn flatten_search_results<'a>(
	commit_list: &mut HashSet<&'a IncludedCommit<'a>>,
	jira_ticket_list: &mut HashSet<&'a str>,
	included_commit: &'a IncludedCommit<'a>,
) {
	// Add the commit to the list
	commit_list.insert(included_commit);

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

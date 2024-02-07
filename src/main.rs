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
mod clipboard;
mod collection;
mod constants;
mod index;
mod multi_writer;
mod search;
mod util;
mod writing;

// Uses
use std::{
	collections::{HashMap, HashSet},
	io::{stdout, Write},
	str::from_utf8,
};

use anyhow::{Context, Result};
use clap::parser::ValuesRef;
use shell_words::split as split_shell_words;

use crate::{
	cli::build_cli,
	clipboard::copy_str_to_clipboard,
	collection::{get_complete_commit_list, Commit},
	index::Index,
	multi_writer::MultiWriter,
	search::{
		build_commit_inclusion_tree,
		flatten_inclusion_tree,
		get_branches_containing,
		get_search_results,
		IncludedCommit,
	},
	util::sortable_jira_ticket,
	writing::{write_to_bin, write_to_markdown},
};

// Constants
const NO_JIRA_TICKET_STR: &str = "*No Jira Ticket*";
const MERGE_COMMIT_MARKER_STR: &str = " (M)";

// Entry Point
fn main() -> Result<()> {
	let cli_definition = build_cli();
	let subcommand_matches = cli_definition.get_matches();

	// Set up the multi-writer
	let mut stdout_writer = stdout();
	let mut string_output_raw = Vec::new();
	let mut multi_writer = MultiWriter::new(vec![&mut stdout_writer, &mut string_output_raw]);

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
			let include_merge_commits = *matches
				.get_one::<bool>("include-merge-commits")
				.unwrap_or(&false);
			let include_mentioned_jira_tickets = *matches
				.get_one::<bool>("include-mentioned")
				.unwrap_or(&false);
			let show_commits = *matches.get_one::<bool>("show-commits").unwrap_or(&false);
			let hash_length = *matches
				.get_one::<u32>("hash-length")
				.expect("Clap provides a default value") as usize;
			let ticket_prefix = matches
				.get_one::<String>("ticket-prefix")
				.expect("Clap provides a default value");
			let copy_to_clipboard = *matches
				.get_one::<bool>("copy-to-clipboard")
				.unwrap_or(&false);

			// Print the revspec used
			writeln!(
				&mut multi_writer,
				"Using the following revspec: `{revspec}`"
			)?;

			// Since the filepaths can be provided all in one argument, or separately with
			// multiple arguments, they need to be collected into a single list
			let mut affected_filepaths = Vec::new();
			if let Some(filepath_sets) = affected_filepath_sets {
				affected_filepaths = flatten_string_sets_on_shell_words(filepath_sets)
					.with_context(|| "unable to parse filepath sets")?;
			}

			// Display the filepaths being considered
			if !affected_filepaths.is_empty() {
				writeln!(
					&mut multi_writer,
					"Only considering commits that affected the following filepaths:"
				)?;
				for affected_filepath in &affected_filepaths {
					writeln!(&mut multi_writer, "- `{affected_filepath}`")?;
				}
			}

			// Collect all commits in the repo
			let commits =
				get_complete_commit_list(repo_dir.as_str(), include_mentioned_jira_tickets)
					.with_context(|| "unable to build the complete commit list from the repo")?;

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
			writeln!(&mut multi_writer)?;

			// Group the commits by Jira ticket
			let jira_ticket_groups = group_by_jira_tickets(search_results.as_slice());
			let jira_ticket_total = if jira_ticket_groups.contains_key(&None) {
				jira_ticket_groups.len() - 1
			} else {
				jira_ticket_groups.len()
			};

			// Sort the Jira tickets
			let mut jira_ticket_groups_sorted = jira_ticket_groups.iter().collect::<Vec<_>>();
			jira_ticket_groups_sorted
				.sort_unstable_by_key(|entry| entry.0.map(sortable_jira_ticket));

			// Display the results
			writeln!(
				&mut multi_writer,
				"Jira tickets: ({jira_ticket_total} total)"
			)?;
			display_jira_ticket_commit_list(
				&mut multi_writer,
				jira_ticket_groups_sorted.as_slice(),
				show_commits,
				hash_length,
				ticket_prefix,
			)?;

			// Copy the output to the clipboard if specified
			if copy_to_clipboard {
				copy_str_to_clipboard(from_utf8(string_output_raw.as_slice()).expect(
					"only string values were written to the buffer, so it's guaranteed to be \
					 valid UTF-8",
				))?;
			}
		}
		Some(("compare", matches)) => {
			// Collect the CLI arguments that were provided
			let repo_dir = matches
				.get_one::<String>("repo")
				.expect("Clap ensures the argument is provided");
			let object_a = matches
				.get_one::<String>("object-a")
				.expect("Clap ensures the argument is provided");
			let object_b = matches
				.get_one::<String>("object-b")
				.expect("Clap ensures the argument is provided");
			let affected_filepath_sets = matches.get_many::<String>("filepath");
			let include_merge_commits = *matches
				.get_one::<bool>("include-merge-commits")
				.unwrap_or(&false);
			let include_cherry_picks = *matches
				.get_one::<bool>("include-cherry-picks")
				.unwrap_or(&false);
			let include_mentioned_jira_tickets = *matches
				.get_one::<bool>("include-mentioned")
				.unwrap_or(&false);
			let show_commits = *matches.get_one::<bool>("show-commits").unwrap_or(&false);
			let hash_length = *matches
				.get_one::<u32>("hash-length")
				.expect("Clap provides a default value") as usize;
			let ticket_prefix = matches
				.get_one::<String>("ticket-prefix")
				.expect("Clap provides a default value");
			let copy_to_clipboard = *matches
				.get_one::<bool>("copy-to-clipboard")
				.unwrap_or(&false);

			// Print the objects being compared
			writeln!(
				&mut multi_writer,
				"Comparing the following two references: `{object_a}` against `{object_b}`"
			)?;

			// Since the filepaths can be provided all in one argument, or separately with
			// multiple arguments, they need to be collected into a single list
			let mut affected_filepaths = Vec::new();
			if let Some(filepath_sets) = affected_filepath_sets {
				affected_filepaths = flatten_string_sets_on_shell_words(filepath_sets)
					.with_context(|| "unable to parse filepath sets")?;
			}

			// Display the filepaths being considered
			if !affected_filepaths.is_empty() {
				writeln!(
					&mut multi_writer,
					"Only considering commits that affected the following filepaths:"
				)?;
				for affected_filepath in &affected_filepaths {
					writeln!(&mut multi_writer, "- `{affected_filepath}`")?;
				}
			}

			// Collect all commits in the repo
			let commits =
				get_complete_commit_list(repo_dir.as_str(), include_mentioned_jira_tickets)
					.with_context(|| "unable to build the complete commit list from the repo")?;

			// Build the index
			let index = Index::new(commits.as_slice())?;

			// Perform the searches
			// The `A ^B` syntax basically searches for all commits accessible from
			// object A, that aren't accessible from object B
			let search_revspec_only_on_object_a = format!("\"{object_a}\" ^\"{object_b}\"");
			let mut search_results_only_on_object_a = get_search_results(
				&index,
				repo_dir.as_str(),
				search_revspec_only_on_object_a.as_str(),
				include_merge_commits,
				affected_filepaths.as_slice(),
			)
			.with_context(|| {
				format!(
					"unable to perform the search for items that are on `{object_a}` but not \
					 `{object_b}`"
				)
			})?;

			let search_revspec_only_on_object_b = format!("\"{object_b}\" ^\"{object_a}\"");
			let mut search_results_only_on_object_b = get_search_results(
				&index,
				repo_dir.as_str(),
				search_revspec_only_on_object_b.as_str(),
				include_merge_commits,
				affected_filepaths.as_slice(),
			)
			.with_context(|| {
				format!(
					"unable to perform the search for items that are on `{object_b}` but not \
					 `{object_a}`"
				)
			})?;

			// Filter out cherry-picks and SVN merges between the two objects
			if !include_cherry_picks {
				// Build hash sets from the results to make searching faster
				let search_results_only_on_object_a_hash_set = search_results_only_on_object_a
					.iter()
					.cloned()
					.collect::<HashSet<_>>();
				let search_results_only_on_object_b_hash_set = search_results_only_on_object_b
					.iter()
					.cloned()
					.collect::<HashSet<_>>();

				// Sets for tracking what should be removed from the original sets
				// (to avoid ad-hoc `remove()` calls)
				let mut object_a_removal_set = HashSet::new();
				let mut object_b_removal_set = HashSet::new();

				// Search both sets, removing commits that reference the other
				// This requires three iterations instead of two because we need to clean up
				// object A with the results of object B's search
				// Technically, this does not cover nested cherry-picks (a cherry-pick of a
				// cherry-pick), but this should basically never happen, so it's not worth
				// covering at the moment
				search_results_only_on_object_a.retain(|commit| {
					if commit.commit.is_likely_a_merge {
						for included_commit in &commit.linked_commits {
							if search_results_only_on_object_b_hash_set.contains(included_commit) {
								object_b_removal_set
									.insert(included_commit.commit.git_revision.clone());
								return false;
							}
						}
					}
					true
				});
				search_results_only_on_object_b.retain(|commit| {
					if object_b_removal_set.contains(&commit.commit.git_revision) {
						return false;
					}
					if commit.commit.is_likely_a_merge {
						for included_commit in &commit.linked_commits {
							if search_results_only_on_object_a_hash_set.contains(included_commit) {
								object_a_removal_set
									.insert(included_commit.commit.git_revision.clone());
								return false;
							}
						}
					}
					true
				});
				search_results_only_on_object_a.retain(|commit| {
					if object_a_removal_set.contains(&commit.commit.git_revision) {
						return false;
					}
					true
				});
			}

			// Group the Jira tickets
			let jira_tickets_on_object_a =
				group_by_jira_tickets(search_results_only_on_object_a.as_slice());
			let jira_tickets_on_object_b =
				group_by_jira_tickets(search_results_only_on_object_b.as_slice());

			// Find the intersection and symmetric differences between the sets
			let mut jira_tickets_only_on_object_a = Vec::new();
			let mut jira_tickets_only_on_object_b = Vec::new();
			let mut jira_tickets_on_both_objects = HashMap::new();
			for (jira_ticket, commits) in &jira_tickets_on_object_a {
				if jira_tickets_on_object_b.contains_key(jira_ticket) {
					jira_tickets_on_both_objects.insert(jira_ticket, (Some(commits), None));
				} else {
					jira_tickets_only_on_object_a.push((jira_ticket, commits));
				}
			}
			for (jira_ticket, commits) in &jira_tickets_on_object_b {
				if jira_tickets_on_object_a.contains_key(jira_ticket) {
					let intersection_set =
						jira_tickets_on_both_objects.get_mut(jira_ticket).expect(
							"this should always exist because we traversed the first set already",
						);
					*intersection_set = (intersection_set.0, Some(commits));
				} else {
					jira_tickets_only_on_object_b.push((jira_ticket, commits));
				}
			}

			let jira_tickets_on_both_objects_total =
				if jira_tickets_on_both_objects.contains_key(&None) {
					jira_tickets_on_both_objects.len() - 1
				} else {
					jira_tickets_on_both_objects.len()
				};

			// Sort the sets
			jira_tickets_only_on_object_a
				.sort_unstable_by_key(|entry| entry.0.map(sortable_jira_ticket));
			jira_tickets_only_on_object_b
				.sort_unstable_by_key(|entry| entry.0.map(sortable_jira_ticket));
			let mut jira_tickets_on_both_objects_sorted =
				jira_tickets_on_both_objects.iter().collect::<Vec<_>>();
			jira_tickets_on_both_objects_sorted
				.sort_unstable_by_key(|entry| entry.0.map(sortable_jira_ticket));

			// We do this search here because a binary search on a sorted set is faster
			let jira_tickets_only_on_object_a_total = if jira_tickets_only_on_object_a
				.binary_search_by_key(&&None, |&(jira_ticket, _)| jira_ticket)
				.is_ok()
			{
				jira_tickets_only_on_object_a.len() - 1
			} else {
				jira_tickets_only_on_object_a.len()
			};
			let jira_tickets_only_on_object_b_total = if jira_tickets_only_on_object_b
				.binary_search_by_key(&&None, |&(jira_ticket, _)| jira_ticket)
				.is_ok()
			{
				jira_tickets_only_on_object_b.len() - 1
			} else {
				jira_tickets_only_on_object_b.len()
			};

			// Display the results
			writeln!(&mut multi_writer)?;
			writeln!(
				&mut multi_writer,
				"Jira tickets only on `{object_a}`: ({jira_tickets_only_on_object_a_total} total)"
			)?;
			display_jira_ticket_commit_list(
				&mut multi_writer,
				jira_tickets_only_on_object_a.as_slice(),
				show_commits,
				hash_length,
				ticket_prefix,
			)?;

			writeln!(&mut multi_writer)?;

			writeln!(
				&mut multi_writer,
				"Jira tickets only on `{object_b}`: ({jira_tickets_only_on_object_b_total} total)"
			)?;
			display_jira_ticket_commit_list(
				&mut multi_writer,
				jira_tickets_only_on_object_b.as_slice(),
				show_commits,
				hash_length,
				ticket_prefix,
			)?;

			writeln!(&mut multi_writer)?;

			writeln!(
				&mut multi_writer,
				"Jira tickets on both `{object_a}` and `{object_b}`: \
				 ({jira_tickets_on_both_objects_total} total)"
			)?;
			display_jira_ticket_commit_list_intersection(
				&mut multi_writer,
				jira_tickets_on_both_objects_sorted.as_slice(),
				object_a.as_str(),
				object_b.as_str(),
				show_commits,
				hash_length,
				ticket_prefix,
			)?;

			// Copy the output to the clipboard if specified
			if copy_to_clipboard {
				copy_str_to_clipboard(from_utf8(string_output_raw.as_slice()).expect(
					"only string values were written to the buffer, so it's guaranteed to be \
					 valid UTF-8",
				))?;
			}
		}
		Some(("search", matches)) => {
			// Collect the CLI arguments that were provided
			let repo_dir = matches
				.get_one::<String>("repo")
				.expect("Clap ensures the argument is provided");
			let jira_tickets = matches
				.get_many::<String>("jira-ticket")
				.expect("Clap ensures at least one argument is provided")
				.collect::<Vec<_>>();
			// let search_tags = *matches.get_one::<bool>("search-tags").unwrap_or(&false);
			let include_mentioned_jira_tickets = *matches
				.get_one::<bool>("include-mentioned")
				.unwrap_or(&false);
			let hash_length = *matches
				.get_one::<u32>("hash-length")
				.expect("Clap provides a default value") as usize;
			let ticket_prefix = matches
				.get_one::<String>("ticket-prefix")
				.expect("Clap provides a default value");
			let copy_to_clipboard = *matches
				.get_one::<bool>("copy-to-clipboard")
				.unwrap_or(&false);

			// Print the search criteria
			writeln!(
				&mut multi_writer,
				"Searching for all locations where any commits were merged for the following:"
			)?;
			for jira_ticket in &jira_tickets {
				writeln!(&mut multi_writer, "- {ticket_prefix}{jira_ticket}")?;
			}

			writeln!(&mut multi_writer)?;

			// Collect all commits in the repo
			let commits =
				get_complete_commit_list(repo_dir.as_str(), include_mentioned_jira_tickets)
					.with_context(|| "unable to build the complete commit list from the repo")?;

			// Build the index
			let index = Index::new(commits.as_slice())?;

			// Find commits that belong to the ticket directly
			// This is an expensive operation, but building it into the index would likely
			// be even worse since we only search once
			let direct_commits = commits
				.iter()
				.filter(|commit| {
					jira_tickets
						.iter()
						.any(|jira_ticket| commit.jira_tickets.contains(jira_ticket))
				})
				.collect::<Vec<_>>();

			// Find all merges of those commits
			let back_reference_inclusion_tree =
				build_commit_inclusion_tree(&index, direct_commits.as_slice(), false, true)
					.with_context(|| "unable to process direct commit results")?;

			// Display the back-reference inclusion tree
			writeln!(
				&mut multi_writer,
				"Commit list being searched, with commits that merge them elsewhere as \
				 sub-entries:"
			)?;
			display_commit_reference_tree(
				&mut multi_writer,
				back_reference_inclusion_tree.as_slice(),
				0,
				hash_length,
			)?;

			writeln!(&mut multi_writer)?;

			// Find all branches (and tags, if requested) that contain any of those commits
			// This yields a list of locations per commit, which then need to be transposed
			// into a list of commits per location
			// A `Vec` is used here instead of a `HashSet` to preserve the order of the
			// commits
			let flattened_inclusion_tree =
				flatten_inclusion_tree(back_reference_inclusion_tree.as_slice());
			let mut commits_per_branch: HashMap<String, Vec<&Commit>> = HashMap::new();
			for commit in flattened_inclusion_tree {
				let branches_containing_commit =
					get_branches_containing(repo_dir, commit.git_revision.as_str()).with_context(
						|| "unable to get the list of branches containing a commit",
					)?;
				for branch in branches_containing_commit {
					commits_per_branch
						.entry(branch)
						.and_modify(|commit_set| commit_set.push(commit))
						.or_insert_with(|| vec![commit]);
				}
			}

			// Group those locations by the commits they contain
			let mut branches_per_commit_set: HashMap<Vec<&Commit>, Vec<String>> = HashMap::new();
			for (branch, commit_set) in commits_per_branch {
				branches_per_commit_set
					.entry(commit_set)
					.and_modify(|branch_list| branch_list.push(branch.clone()))
					.or_insert_with(|| vec![branch]);
			}

			// Sort the branch lists
			// The branches are collected into the sets without a particular order, and
			// sorting them makes the final output more readable
			let mut branches_per_commit_set_ordered =
				branches_per_commit_set.drain().collect::<Vec<_>>();
			for (_, branch_set) in &mut branches_per_commit_set_ordered {
				branch_set.sort();
			}

			// Sort the list of branch sets to put the sets with the most branches near the
			// top
			branches_per_commit_set_ordered.sort_by_key(|(_, branch_set)| branch_set.len());
			branches_per_commit_set_ordered.reverse();

			// Display the branches where each specific set of commits is
			writeln!(&mut multi_writer, "Results:")?;
			for (index, (commit_set, branch_set)) in
				branches_per_commit_set_ordered.iter().enumerate()
			{
				writeln!(&mut multi_writer, "- Set {index}:")?;
				writeln!(&mut multi_writer, "\t- Commits:")?;
				display_commit_set(&mut multi_writer, commit_set.as_slice(), 2, hash_length)?;
				writeln!(&mut multi_writer, "\t- Branches:")?;
				for branch in branch_set {
					writeln!(&mut multi_writer, "\t\t- `{branch}`")?;
				}
			}

			// Copy the output to the clipboard if specified
			if copy_to_clipboard {
				copy_str_to_clipboard(from_utf8(string_output_raw.as_slice()).expect(
					"only string values were written to the buffer, so it's guaranteed to be \
					 valid UTF-8",
				))?;
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
				.with_context(|| "unable to build the complete commit list from the repo")?;

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

/// Flattens string sets based on shell "words".
///
/// For example: `"abc def", "ghi"` -> `"abc", "def", "ghi"`
fn flatten_string_sets_on_shell_words(string_sets: ValuesRef<String>) -> Result<Vec<String>> {
	let mut flattened_set = Vec::with_capacity(string_sets.len());
	for string_set in string_sets {
		flattened_set.extend(
			split_shell_words(string_set.as_str())
				.with_context(|| format!("unable to split set: {string_set}"))?,
		);
	}
	flattened_set.sort_unstable();

	Ok(flattened_set)
}

/// Group a set of included commits by Jira ticket.
fn group_by_jira_tickets<'a>(
	included_commits: &'a [IncludedCommit<'a>],
) -> HashMap<Option<&'a str>, Vec<IncludedCommit<'a>>> {
	let mut jira_ticket_groups = HashMap::new();

	for included_commit in included_commits {
		// The `clone` calls here are a little ugly, but the `IncludedCommit` struct
		// basically just holds references anyway, so cloning it is cheap
		if included_commit.commit.jira_tickets.is_empty() {
			jira_ticket_groups
				.entry(None)
				.and_modify(|ticket_commits: &mut Vec<IncludedCommit>| {
					ticket_commits.push(included_commit.clone());
				})
				.or_insert_with(|| vec![included_commit.clone()]);
		} else {
			for jira_ticket in &included_commit.commit.jira_tickets {
				jira_ticket_groups
					.entry(Some(jira_ticket.as_str()))
					.and_modify(|ticket_commits: &mut Vec<IncludedCommit>| {
						ticket_commits.push(included_commit.clone());
					})
					.or_insert_with(|| vec![included_commit.clone()]);
			}
		}
	}

	jira_ticket_groups
}

/// Displays the simple list of Jira tickets, optionally with commit
/// information.
#[allow(clippy::ref_option_ref)]
fn display_jira_ticket_commit_list(
	multi_writer: &mut MultiWriter,
	jira_tickets: &[(&Option<&str>, &Vec<IncludedCommit>)],
	show_commits: bool,
	hash_length: usize,
	ticket_prefix: &str,
) -> Result<()> {
	for (jira_ticket_option, commits) in jira_tickets {
		let jira_ticket = if let Some(ticket) = jira_ticket_option {
			format!("{ticket_prefix}{ticket}")
		} else {
			NO_JIRA_TICKET_STR.to_owned()
		};
		if show_commits {
			writeln!(multi_writer, "- {jira_ticket}:")?;
			display_commit_reference_tree(multi_writer, commits.as_slice(), 1, hash_length)?;
		} else {
			writeln!(multi_writer, "- {jira_ticket} ({})", commits.len())?;
		}
	}

	Ok(())
}

/// Displays the list of Jira tickets for the intersection between two objects'
/// lists.
///
/// The counterpart of `display_jira_ticket_commit_list`.
///
/// The reason this is in its own function despite only being used once, is so
/// that it can be updated in tandem with its counterpart.
#[allow(clippy::ref_option_ref, clippy::type_complexity)]
fn display_jira_ticket_commit_list_intersection(
	multi_writer: &mut MultiWriter,
	jira_ticket_intersection: &[(
		&&Option<&str>,
		&(Option<&Vec<IncludedCommit>>, Option<&Vec<IncludedCommit>>),
	)],
	object_a: &str,
	object_b: &str,
	show_commits: bool,
	hash_length: usize,
	ticket_prefix: &str,
) -> Result<()> {
	for (jira_ticket_option, (commits_object_a, commits_object_b)) in jira_ticket_intersection {
		let jira_ticket = if let Some(ticket) = jira_ticket_option {
			format!("{ticket_prefix}{ticket}")
		} else {
			NO_JIRA_TICKET_STR.to_owned()
		};
		let commits_object_a = commits_object_a
			.expect("the Option types are just present for the population stage of the process");
		let commits_object_b = commits_object_b
			.expect("the Option types are just present for the population stage of the process");
		if show_commits {
			writeln!(multi_writer, "- {jira_ticket}:")?;
			writeln!(multi_writer, "\t- On `{object_a}`:")?;
			display_commit_reference_tree(
				multi_writer,
				commits_object_a.as_slice(),
				2,
				hash_length,
			)?;
			writeln!(multi_writer, "\t- On `{object_b}`:")?;
			display_commit_reference_tree(
				multi_writer,
				commits_object_b.as_slice(),
				2,
				hash_length,
			)?;
		} else {
			writeln!(
				multi_writer,
				"- {jira_ticket} ({} : {})",
				commits_object_a.len(),
				commits_object_b.len()
			)?;
		}
	}

	Ok(())
}

/// Displays the commit reference tree for a set of commits.
fn display_commit_reference_tree(
	multi_writer: &mut MultiWriter,
	included_commits: &[IncludedCommit],
	indentation: u32,
	hash_length: usize,
) -> Result<()> {
	for included_commit in included_commits {
		// Print the indentation
		for _ in 0..indentation {
			write!(multi_writer, "\t")?;
		}

		// Print the commit revision
		writeln!(
			multi_writer,
			"- `{}`{}",
			&included_commit.commit.git_revision[0..hash_length],
			if included_commit.commit.is_likely_a_merge {
				MERGE_COMMIT_MARKER_STR
			} else {
				""
			}
		)?;

		// Recurse over the referenced commits
		display_commit_reference_tree(
			multi_writer,
			included_commit.linked_commits.as_slice(),
			indentation + 1,
			hash_length,
		)?;
	}

	Ok(())
}

/// Displays a set of commits.
fn display_commit_set(
	multi_writer: &mut MultiWriter,
	commits: &[&Commit],
	indentation: u32,
	hash_length: usize,
) -> Result<()> {
	for commit in commits {
		// Print the indentation
		for _ in 0..indentation {
			write!(multi_writer, "\t")?;
		}

		// Print the commit revision
		writeln!(
			multi_writer,
			"- `{}`{}",
			&commit.git_revision[0..hash_length],
			if commit.is_likely_a_merge {
				MERGE_COMMIT_MARKER_STR
			} else {
				""
			}
		)?;
	}

	Ok(())
}

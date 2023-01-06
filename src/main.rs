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
mod parsing;
mod util;
mod writing;

// Uses
use anyhow::{Context, Result};

use crate::{
	cli::build_cli,
	parsing::get_repo_revision_maps,
	writing::{write_to_bin, write_to_markdown, write_to_markdown_basic},
};

// Entry Point
fn main() -> Result<()> {
	let cli_definition = build_cli();
	let matches = cli_definition.get_matches();
	let repo_dir = matches
		.get_one::<String>("repo")
		.expect("Clap ensures the argument is provided");

	let revision_maps = get_repo_revision_maps(repo_dir.as_str())
		.with_context(|| "unable to get the repo revision maps")?;

	if let Some(path) = matches.get_one::<String>("binary") {
		write_to_bin(path, revision_maps.as_slice())
			.with_context(|| "unable to write the revision map to binary")?;
	} else if let Some(path) = matches.get_one::<String>("markdown") {
		let git_url_base = matches
			.get_one::<String>("git-url-base")
			.expect("Clap ensures the argument is provided");
		write_to_markdown(path, git_url_base.as_str(), revision_maps.as_slice())
			.with_context(|| "unable to write the revision map to markdown")?;
	} else if let Some(path) = matches.get_one::<String>("markdown-basic") {
		write_to_markdown_basic(path, revision_maps.as_slice())
			.with_context(|| "unable to write the revision map to markdown")?;
	} else {
		unreachable!("Clap ensures exactly one output path is provided");
	};

	Ok(())
}

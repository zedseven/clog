//! Provides the CLI for the program.

// Uses
use clap::{builder::NonEmptyStringValueParser, value_parser, Arg, ArgAction, ArgGroup, Command};

use crate::constants::{APPLICATION_PROPER_NAME, SHA1_HASH_ASCII_LENGTH};

// Constants
const HELP_TEMPLATE: &str = "\
{before-help}{name} {version}
{author-with-newline}{about-with-newline}
{usage-heading} {usage}

{all-args}{after-help}
";

/// Builds the command-line interface.
pub fn build_cli() -> Command {
	let repo_arg = Arg::new("repo")
		.short('r')
		.long("repo")
		.visible_alias("repository")
		.visible_alias("git-repo")
		.num_args(1)
		.default_value(".")
		.action(ArgAction::Set)
		.value_name("PATH")
		.help("The path to the Git repository to read from.")
		.value_parser(NonEmptyStringValueParser::new());
	let hash_length_arg = Arg::new("hash-length")
		.short('l')
		.long("hash-length")
		.visible_alias("length")
		.num_args(1)
		.default_value("8")
		.action(ArgAction::Set)
		.value_name("LENGTH")
		.help("The number of characters to abbreviate Git revision hashes to when displayed.")
		.value_parser(value_parser!(u32).range(6..=SHA1_HASH_ASCII_LENGTH as i64));

	let filepath_arg = Arg::new("filepath")
		.short('p')
		.long("filepath")
		.visible_short_alias('p')
		.visible_alias("file")
		.visible_alias("dir")
		.visible_alias("directory")
		.visible_alias("path")
		.visible_alias("affected")
		.num_args(1)
		.action(ArgAction::Append)
		.value_name("RELATIVE_PATH")
		.help(
			"Filter the results to only commits that affected the specified \
			 filepaths/directories.\nThe paths should be relative to the repository root. \
			 Multiple paths can be provided, separated by spaces, or this argument can be \
			 provided multiple times.",
		)
		.value_parser(NonEmptyStringValueParser::new());
	let include_merge_commits_arg = Arg::new("include-merge-commits")
		.short('M')
		.long("include-merge-commits")
		.visible_alias("include-merges")
		.visible_alias("merge-commits")
		.num_args(0..=1)
		.default_value("false")
		.default_missing_value("true")
		.action(ArgAction::Set)
		.value_name("TRUE/FALSE")
		.value_parser(value_parser!(bool))
		.help(
			"Include merge commits in the results.\nThis is off by default because they don't add \
			 much to the resulting data, and tend to bloat the results.",
		);
	let include_mentioned_arg = Arg::new("include-mentioned")
		.short('m')
		.long("include-mentioned")
		.visible_alias("mentioned")
		.visible_alias("mentioned-tickets")
		// Not a fan of using "Jira" as a synonym for "ticket", but it makes sense as an
		// alias
		.visible_alias("mentioned-jiras")
		.num_args(0..=1)
		.default_value("false")
		.default_missing_value("true")
		.action(ArgAction::Set)
		.value_name("TRUE/FALSE")
		.value_parser(value_parser!(bool))
		.help(
			"Include Jira tickets that were mentioned anywhere in the commit message, instead of \
			 just at the beginning. Please note that if using this feature, the same commit may \
			 be counted in multiple Jira tickets.",
		);
	let show_commits_arg = Arg::new("show-commits")
		.short('c')
		.long("show-commits")
		.visible_alias("commits")
		.visible_alias("commit-hashes")
		.num_args(0..=1)
		.default_value("false")
		.default_missing_value("true")
		.action(ArgAction::Set)
		.value_name("TRUE/FALSE")
		.value_parser(value_parser!(bool))
		.help(
			"Include commit hash information in the display. This option is disabled by default \
			 because it makes the results too noisy and does not help unless checking the commit \
			 information for technical reasons is required.",
		);
	let ticket_prefix_arg = Arg::new("ticket-prefix")
		.short('P')
		.long("ticket-prefix")
		.visible_alias("jira-ticket-prefix")
		.visible_alias("jira-prefix")
		.num_args(1)
		.default_value("")
		.action(ArgAction::Set)
		.value_name("PREFIX")
		.help(
			"The prefix to apply to Jira tickets in the output. This is a convenience feature to \
			 make the output more directly-usable with external tools, like turning each ticket \
			 into a tag in Obsidian.",
		);
	let copy_to_clipboard_arg = Arg::new("copy-to-clipboard")
		.short('C')
		.long("copy-to-clipboard")
		.visible_alias("clipboard")
		.visible_alias("copy")
		.num_args(0..=1)
		.default_value("false")
		.default_missing_value("true")
		.action(ArgAction::Set)
		.value_name("TRUE/FALSE")
		.value_parser(value_parser!(bool))
		.help(format!(
			"Copy the output to the clipboard automatically, which makes it easy to paste \
			 elsewhere with the correct formatting.\nNote that on some operating systems (Linux), \
			 the clipboard contents are lost when the application that set them exits. To avoid \
			 this, {APPLICATION_PROPER_NAME} will wait until Enter is pressed before exiting so \
			 that the contents can be pasted where they're needed.",
		));

	let list_subcommand = Command::new("list")
		.about("Generates lists of information based on a provided revspec.")
		.arg_required_else_help(true)
		.arg(repo_arg.clone())
		.arg(
			Arg::new("revspec")
				.num_args(1)
				.action(ArgAction::Set)
				.value_name("REVSPEC")
				.required(true)
				.help(format!(
					"The revision(s)/reference(s) to inspect. This is passed verbatim to `git \
					 log`.\nFor a simple revision range, use `A..B` (without quotes) where A \
					 comes before B in the history.\nFor a list of all changes to be merged \
					 together (on both branches), use `A...B` where A and B are the tips of the \
					 two branches being merged. Note the 3 dots in this case, instead of 2.\nFor \
					 a list of all changes that will be merged into another branch, use `A ^B` \
					 where B is the base branch, and A is the branch to be merged into B.\nFor \
					 more information, review: {}",
					"https://git-scm.com/book/en/v2/Git-Tools-Revision-Selection"
				))
				.value_parser(NonEmptyStringValueParser::new()),
		)
		.arg(filepath_arg.clone())
		.arg(include_merge_commits_arg.clone())
		.arg(include_mentioned_arg.clone())
		.arg(show_commits_arg.clone())
		.arg(hash_length_arg.clone())
		.arg(ticket_prefix_arg.clone())
		.arg(copy_to_clipboard_arg.clone());

	let compare_subcommand = Command::new("compare")
		.about(
			"Compares two objects and generates a comprehensive list of differences (as best as \
			 possible).",
		)
		.arg_required_else_help(true)
		.arg(repo_arg.clone())
		.arg(
			Arg::new("object-a")
				.num_args(1)
				.action(ArgAction::Set)
				.value_name("OBJECT_A")
				.required(true)
				.help("The first reference to compare.")
				.value_parser(NonEmptyStringValueParser::new()),
		)
		.arg(
			Arg::new("object-b")
				.num_args(1)
				.action(ArgAction::Set)
				.value_name("OBJECT_B")
				.required(true)
				.help("The second reference to compare.")
				.value_parser(NonEmptyStringValueParser::new()),
		)
		.arg(filepath_arg)
		.arg(include_merge_commits_arg)
		.arg(
			Arg::new("include-cherry-picks")
				.long("include-cherry-picks")
				.visible_alias("cherry-picks")
				.num_args(0..=1)
				.default_value("false")
				.default_missing_value("true")
				.action(ArgAction::Set)
				.value_name("TRUE/FALSE")
				.value_parser(value_parser!(bool))
				.help(
					"When this is false (default), the results will be filtered so that \
					 cherry-picks of commits on the other object are removed. This cleans up the \
					 results by removing changes that are on both objects, just under different \
					 commits.\nUnfortunately, to do this, a heuristic is used that is not \
					 perfect. As a result, this option provides the ability to disable the \
					 functionality in case of issues.",
				),
		)
		.arg(include_mentioned_arg)
		.arg(show_commits_arg)
		.arg(hash_length_arg.clone())
		.arg(ticket_prefix_arg.clone())
		.arg(copy_to_clipboard_arg);

	let revmap_subcommand = Command::new("revmap")
		.visible_alias("build-revmap") // Since `clog` started as `build-revmap`
		.visible_alias("svn-revmap")
		.about(
			"Generates an SVN to Git revision map based on the metadata in commit messages, \
			 provided by `git-svn` with the `--metadata` flag.",
		)
		.arg_required_else_help(true)
		.arg(repo_arg)
		.group(
			ArgGroup::new("outputs")
				.args(["binary", "markdown"])
				.required(true)
				.multiple(true),
		)
		.arg(
			Arg::new("binary")
				.short('b')
				.long("binary")
				.visible_alias("bin")
				.num_args(1)
				.action(ArgAction::Set)
				.value_name("PATH")
				.help("Write the results to a binary file at PATH.")
				.value_parser(NonEmptyStringValueParser::new()),
		)
		.arg(
			Arg::new("markdown")
				.short('m')
				.long("markdown")
				.visible_alias("md")
				.num_args(1)
				.action(ArgAction::Set)
				.value_name("PATH")
				.help("Write the results to a Markdown file at PATH.")
				.value_parser(NonEmptyStringValueParser::new()),
		)
		.arg(hash_length_arg);

	Command::new(APPLICATION_PROPER_NAME)
		.version(env!("CARGO_PKG_VERSION"))
		.author(env!("CARGO_PKG_AUTHORS"))
		.about(env!("CARGO_PKG_DESCRIPTION"))
		.help_template(HELP_TEMPLATE)
		.arg_required_else_help(true)
		.help_expected(true)
		.subcommand(list_subcommand)
		.subcommand(compare_subcommand)
		.subcommand(revmap_subcommand)
}

//! Provides the CLI for the program.

// Uses
use clap::{builder::NonEmptyStringValueParser, value_parser, Arg, ArgAction, ArgGroup, Command};

use crate::constants::SHA1_HASH_ASCII_LENGTH;

// Constants
pub const APPLICATION_PROPER_NAME: &str = "CLog";
pub const APPLICATION_BIN_NAME: &str = env!("CARGO_PKG_NAME");
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
		.help("The number of characters to abbreviate Git revision hashes to.")
		.value_parser(value_parser!(u32).range(6..=SHA1_HASH_ASCII_LENGTH as i64));

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
		.arg(
			Arg::new("filepath")
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
				.value_parser(NonEmptyStringValueParser::new()),
		)
		.arg(
			Arg::new("include-merges")
				.short('m')
				.long("include-merges")
				.visible_alias("merges")
				.visible_alias("merge-commits")
				.visible_alias("include-merge-commits")
				.num_args(0..=1)
				.default_value("false")
				.default_missing_value("true")
				.action(ArgAction::Set)
				.value_name("TRUE/FALSE")
				.value_parser(value_parser!(bool))
				.help(
					"Include merge commits in the results.\nThis is off by default because they \
					 don't add much to the resulting data, and tend to bloat the results.",
				),
		)
		.arg(
			Arg::new("include-mentioned")
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
					"Include Jira tickets that were mentioned anywhere in the commit message, \
					 instead of just at the beginning.",
				),
		)
		.arg(
			Arg::new("flatten")
				.short('f')
				.long("flatten")
				.visible_alias("flatten-results")
				.num_args(0..=1)
				.default_value("false")
				.default_missing_value("true")
				.action(ArgAction::Set)
				.value_name("TRUE/FALSE")
				.value_parser(value_parser!(bool))
				.help(
					"Flatten the results so there is no nesting of commits or Jira tickets. The \
					 output will be 2 distinct lists of information.\nNote: The flattened lists \
					 will include more Jira tickets, because the Jira tickets of the referenced \
					 commits will also be included. (whereas, with the normal behaviour, \
					 referenced commits' Jira tickets are ignored)",
				),
		)
		.arg(hash_length_arg.clone());

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
		.subcommand(revmap_subcommand)
}

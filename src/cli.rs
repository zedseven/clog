//! Provides the CLI for the program.

// Uses
use clap::{builder::NonEmptyStringValueParser, Arg, ArgAction, ArgGroup, Command};

// Constants
pub const APPLICATION_PROPER_NAME: &str = "Merged Lists";
pub const APPLICATION_BIN_NAME: &str = env!("CARGO_PKG_NAME");

/// Builds the command-line interface.
pub fn build_cli() -> Command {
	let repo_arg = Arg::new("repo")
		.long("repo")
		.visible_alias("repository")
		.visible_alias("git-repo")
		.num_args(1)
		.default_value(".")
		.action(ArgAction::Set)
		.value_name("PATH")
		.help("The path to the Git repository to read from.")
		.value_parser(NonEmptyStringValueParser::new());

	let list_subcommand = Command::new("list")
        .about("Generates lists of information based on a provided revspec.")
        .arg_required_else_help(true)
		.arg(repo_arg.clone())
		.arg(
			Arg::new("revspec")
				.long("revspec")
				.visible_alias("revision")
				.visible_alias("spec")
				.visible_alias("range")
				.num_args(1)
				.action(ArgAction::Set)
				.value_name("REVSPEC")
				.required(true)
				.help(
					"The revision(s)/reference(s) to inspect. This is passed verbatim to `git \
					 log`.\nReview https://git-scm.com/book/en/v2/Git-Tools-Revision-Selection for more information.",
				)
				.value_parser(NonEmptyStringValueParser::new()),
		);

	let revmap_subcommand = Command::new("revmap")
		.visible_alias("svn-revmap")
		.about(
			"Generates an SVN to Git revision map based on the metadata in commit messages, \
			 provided by `git-svn` with the `--metadata` flag.",
		)
		.arg_required_else_help(true)
		.arg(repo_arg)
		.group(
			ArgGroup::new("outputs")
				.args(["binary", "markdown", "markdown-basic"])
				.required(true),
		)
		.arg(
			Arg::new("binary")
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
				.long("markdown")
				.visible_alias("md")
				.num_args(1)
				.action(ArgAction::Set)
				.value_name("PATH")
				.requires("git-url-base")
				.help("Write the results to a Markdown file at PATH.")
				.value_parser(NonEmptyStringValueParser::new()),
		)
		.arg(
			Arg::new("markdown-basic")
				.long("markdown-basic")
				.num_args(1)
				.action(ArgAction::Set)
				.value_name("PATH")
				.help(
					"Write the results to a Markdown file at PATH. This is the basic version, \
					 without repository links, to save space.",
				)
				.value_parser(NonEmptyStringValueParser::new()),
		)
		.arg(
			Arg::new("git-url-base")
				.long("git-url-base")
				.visible_alias("git-url")
				.num_args(1)
				.action(ArgAction::Set)
				.value_name("URL")
				.requires("markdown")
				.help(
					"The URL base used for linking to specific commits. The base is expected to \
					 include a trailing slash.",
				)
				.value_parser(NonEmptyStringValueParser::new()),
		);

	Command::new(APPLICATION_PROPER_NAME)
		.version(env!("CARGO_PKG_VERSION"))
		.author(env!("CARGO_PKG_AUTHORS"))
		.about(env!("CARGO_PKG_DESCRIPTION"))
		.arg_required_else_help(true)
		.help_expected(true)
		.subcommand(list_subcommand)
		.subcommand(revmap_subcommand)
}

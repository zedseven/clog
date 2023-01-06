// Provides the CLI for the program.

// Uses
use clap::{builder::NonEmptyStringValueParser, Arg, ArgAction, Command};

// Constants
pub const APPLICATION_PROPER_NAME: &str = "Build Revision Map";
pub const APPLICATION_BIN_NAME: &str = env!("CARGO_PKG_NAME");

/// Builds the command-line interface.
pub fn build_cli() -> Command {
	Command::new(APPLICATION_PROPER_NAME)
		.version(env!("CARGO_PKG_VERSION"))
		.author(env!("CARGO_PKG_AUTHORS"))
		.about(env!("CARGO_PKG_DESCRIPTION"))
		.arg_required_else_help(true)
		.help_expected(true)
		.arg(
			Arg::new("repo")
				.long("repo")
				.visible_alias("repository")
				.visible_alias("git-repo")
				.num_args(1)
				.default_value(".")
				.action(ArgAction::Set)
				.value_name("PATH")
				.requires("outputs")
				.help("The path to the Git repository to read from.")
				.value_parser(NonEmptyStringValueParser::new()),
		)
		.arg(
			Arg::new("binary")
				.group("outputs")
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
				.group("outputs")
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
				.group("outputs")
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
		)
}

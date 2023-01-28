//! The module that provides utility functions.

// Uses
use std::{
	num::ParseIntError,
	process::Command,
	result::Result as StdResult,
	str::from_utf8 as str_from_utf8,
};

use anyhow::{anyhow, Context, Result};

/// Runs a provided command and returns the stdout in UTF-8.
pub fn run_command(mut command: Command) -> Result<String> {
	// Run the command
	let command_result = command
		.output()
		.with_context(|| "unable to run the command")?;
	if !command_result.status.success() {
		return Err(anyhow!(
			"command failed: {:?}",
			command_result.status.code()
		));
	}

	// Convert the command output into a usable string of UTF-8
	str_from_utf8(&command_result.stdout)
		.with_context(|| "unable to parse command output as UTF-8")
		.map(ToOwned::to_owned)
}

/// Swaps the nesting order of a `Result<Option<T>, E>` to an `Option<Result<T,
/// E>>`.
pub fn inside_out_result<T, E>(result: Result<Option<T>, E>) -> Option<Result<T, E>> {
	match result {
		Ok(Some(t)) => Some(Ok(t)),
		Ok(None) => None,
		Err(e) => Some(Err(e)),
	}
}

/// Swaps the nesting order of an `Option<Result<T, E>>` to a `Result<Option<T>,
/// E>`.
pub fn inside_out_option<T, E>(option: Option<Result<T, E>>) -> Result<Option<T>, E> {
	match option {
		Some(Ok(t)) => Ok(Some(t)),
		Some(Err(e)) => Err(e),
		None => Ok(None),
	}
}

/// Converts a vector to a fixed-size array.
pub fn vec_to_arr<T, const N: usize>(vec: Vec<T>) -> [T; N] {
	vec.try_into()
		.unwrap_or_else(|v: Vec<T>| panic!("expected a Vec of length {} but it was {}", N, v.len()))
}

/// Parses a string containing only hexadecimal characters into a vector of
/// bytes.
///
/// Sourced from: <https://stackoverflow.com/a/52992629>
pub fn parse_hex_str(s: &str) -> StdResult<Vec<u8>, ParseIntError> {
	(0..s.len())
		.step_by(2)
		.map(|i| u8::from_str_radix(&s[i..i + 2], 16))
		.collect()
}

/// Converts a slice of bytes into a string.
pub fn bytes_to_str(bytes: &[u8]) -> String {
	fn nibble_to_char(num: u8) -> char {
		(match num {
			0..=9 => num + 0x30,
			10..=15 => (num - 10) + 0x61,
			_ => unreachable!("there should be nothing higher than 0xf"),
		}) as char
	}

	let mut result = String::with_capacity(bytes.len() * 2);
	for &byte in bytes {
		result.push(nibble_to_char((0b1111_0000 & byte) >> 4));
		result.push(nibble_to_char(0b0000_1111 & byte));
	}
	result
}

/// Takes a Jira ticket and returns it in a format that can be used as a sorting
/// key to avoid an ASCII sort.
pub fn sortable_jira_ticket(jira_ticket: &str) -> (&str, u32) {
	// Split on the hyphen
	let (project, issue) = jira_ticket.split_once('-').expect(
		"all Jira tickets should have 1 hyphen separating the project from the issue number",
	);

	// Parse the issue number as an integer
	let issue_num = issue
		.parse::<u32>()
		.expect("all issue numbers should be numeric");

	// Return the pair, so they can be used as a sorting key
	(project, issue_num)
}

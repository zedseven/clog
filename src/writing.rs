// Uses
use std::{fs::File, io::Write, path::Path};

use anyhow::{Context, Result};

use crate::util::{bytes_to_str, parse_hex_str};

/// Based on: <https://github.com/hexmode/git-1/blob/master/perl/Git/SVN.pm#L2170>
pub fn write_to_bin<P>(path: P, revision_map: &[(u32, &str, &str)]) -> Result<()>
where
	P: AsRef<Path>,
{
	let mut output_bin = Vec::new();

	for revision_map in revision_map {
		let svn_bytes = revision_map.0.to_be_bytes();
		output_bin.extend_from_slice(&svn_bytes);

		let git_bytes = parse_hex_str(revision_map.2)
			.expect("this should always be valid hex because it comes from Git directly");
		output_bin.extend_from_slice(git_bytes.as_slice());
	}

	let mut output_file = File::create(path).with_context(|| "unable to open path for writing")?;
	output_file
		.write_all(output_bin.as_slice())
		.with_context(|| "unable to write bytes to the file")
}

pub fn write_to_markdown<P>(
	path: P,
	revision_map: &[(u32, &str, &str)],
	hash_length: usize,
) -> Result<()>
where
	P: AsRef<Path>,
{
	let mut output_str = String::new();

	for revision_map in revision_map {
		output_str.push_str(
			format!(
				"- `{}` -> `{}` (`{}`)\n",
				revision_map.0,
				&revision_map.2[0..hash_length],
				revision_map.1,
			)
			.as_str(),
		);
	}

	let mut output_file = File::create(path).with_context(|| "unable to open path for writing")?;
	output_file
		.write_all(output_str.as_bytes())
		.with_context(|| "unable to write bytes to the file")
}

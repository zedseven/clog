// Uses
use std::{fs::File, io::Write, path::Path};

use anyhow::{Context, Result};

use crate::{parsing::RevisionMapping, util::bytes_to_str};

/// Based on: <https://github.com/hexmode/git-1/blob/master/perl/Git/SVN.pm#L2170>
pub fn write_to_bin<P>(path: P, revision_maps: &[RevisionMapping]) -> Result<()>
where
	P: AsRef<Path>,
{
	let mut output_bin = Vec::new();

	for revision_map in revision_maps {
		let svn_bytes = revision_map.svn_revision.to_be_bytes();
		output_bin.extend_from_slice(&svn_bytes);

		output_bin.extend_from_slice(&revision_map.git_revision);
	}

	let mut output_file = File::create(path).with_context(|| "unable to open path for writing")?;
	output_file
		.write_all(output_bin.as_slice())
		.with_context(|| "unable to write bytes to the file")
}

pub fn write_to_markdown<P>(
	path: P,
	git_url_base: &str,
	revision_maps: &[RevisionMapping],
) -> Result<()>
where
	P: AsRef<Path>,
{
	let mut output_str = String::new();

	for revision_map in revision_maps {
		let git_revision_string = bytes_to_str(&revision_map.git_revision);
		let git_revision_short_string = bytes_to_str(&revision_map.git_revision[0..4]);

		output_str.push_str(
			format!(
				"- {}: [{}]({}) -> [{}]({}{})\n",
				revision_map.svn_revision,
				revision_map.svn_url,
				revision_map.svn_url,
				git_revision_short_string.as_str(),
				git_url_base,
				git_revision_string.as_str(),
			)
			.as_str(),
		);
	}

	let mut output_file = File::create(path).with_context(|| "unable to open path for writing")?;
	output_file
		.write_all(output_str.as_bytes())
		.with_context(|| "unable to write bytes to the file")
}

//! The module for indexing collected commit data to make it searchable.

// Uses
use std::collections::{BTreeMap, HashMap};

use anyhow::{anyhow, Result};

use crate::collection::Commit;

// Types and Structures
// pub type Sha1Hash = [u8; SHA1_HASH_LENGTH];

#[derive(Debug)]
pub struct Index<'a> {
	pub git_revision_map:        BTreeMap<&'a str, &'a Commit>,
	pub svn_to_git_revision_map: HashMap<u32, &'a str>,
}

impl<'a> Index<'a> {
	pub fn new(commits: &'a [Commit]) -> Result<Self> {
		// Build the lookup maps
		let mut git_revision_map = BTreeMap::new();
		let mut svn_to_git_revision_map = HashMap::new();
		for commit in commits {
			// Cache the Git revision number for partial lookup later
			git_revision_map.insert(commit.git_revision.as_str(), commit);

			// Cache the SVN to Git revision relationship
			if let Some(svn_info) = &commit.svn_info {
				svn_to_git_revision_map.insert(svn_info.svn_revision, commit.git_revision.as_str());
			}
		}

		// Return the completed index
		Ok(Self {
			git_revision_map,
			svn_to_git_revision_map,
		})
	}

	pub fn lookup_git_revision(&self, partial_revision: &str) -> Result<&'a Commit> {
		// This is a little complicated, but it uses the binary tree to quickly find
		// full revisions that match the provided partial one
		let matching_revisions = self
			.git_revision_map
			.range(partial_revision..)
			.take_while(|full_git_revision_entry| {
				full_git_revision_entry.0.starts_with(partial_revision)
			})
			.collect::<Vec<_>>();

		// Handle the different cases for the number of potential matches
		match matching_revisions.len() {
			0 => Err(anyhow!(
				"no matching full revision for the provided partial revision \
				 \"{partial_revision}\""
			)),
			1 => Ok(matching_revisions[0].1),
			_ => Err(anyhow!(
				"multiple matching full revisions for the provided partial revision \
				 \"{partial_revision}\" and no way to tell which is the correct one"
			)),
		}
	}

	pub fn lookup_svn_revision(&self, svn_revision: u32) -> Result<&'a Commit> {
		// Lookup the SVN revision and get the corresponding Git revision
		let git_revision = self
			.svn_to_git_revision_map
			.get(&svn_revision)
			.ok_or_else(|| {
				anyhow!("no matching commit for the provided SVN revision {svn_revision}")
			})?;

		// Get the actual commit for the Git revision
		Ok(self.git_revision_map.get(git_revision).expect(
			"there should always be a Git commit if the entry exists in the SVN to Git revision \
			 map",
		))
	}
}

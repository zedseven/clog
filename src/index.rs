//! The module for indexing collected commit data to make it searchable.

// Uses
use std::collections::{BTreeMap, HashMap};

use anyhow::{anyhow, Result};

use crate::collection::Commit;

// Types and Structures
// pub type Sha1Hash = [u8; SHA1_HASH_LENGTH];

#[derive(Debug)]
pub struct Index<'a> {
	git_revision_map:        BTreeMap<&'a str, &'a Commit>,
	svn_to_git_revision_map: HashMap<u32, &'a str>,
	forward_references:      HashMap<&'a Commit, Vec<&'a Commit>>,
	backward_references:     HashMap<&'a Commit, Vec<&'a Commit>>,
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

		let mut index = Self {
			git_revision_map,
			svn_to_git_revision_map,
			forward_references: HashMap::new(),
			backward_references: HashMap::new(),
		};

		// Build the reference maps using the functionality provided by the first stage
		let mut forward_references: HashMap<&Commit, Vec<&Commit>> = HashMap::new();
		let mut backward_references: HashMap<&Commit, Vec<&Commit>> = HashMap::new();
		for commit in commits {
			// Follow Git revision references
			for git_revision in &commit.referenced_commits.git_commits {
				// Lookup the reference
				if let Ok(referenced_commit) = index.lookup_git_revision(git_revision.as_str()) {
					forward_references
						.entry(commit)
						.and_modify(|referenced_commits| referenced_commits.push(referenced_commit))
						.or_insert_with(|| vec![referenced_commit]);
					backward_references
						.entry(referenced_commit)
						.and_modify(|referencing_commits| referencing_commits.push(commit))
						.or_insert_with(|| vec![referenced_commit]);
				} else {
					#[cfg(debug_assertions)]
					if is_likely_a_real_git_revision(git_revision) {
						eprintln!(
							"[WARNING] Git revision `{git_revision}` referenced by commit `{}` \
							 could not be found.",
							commit.git_revision
						);
					}
				}
			}

			// Follow SVN revision references
			for svn_revision in &commit.referenced_commits.svn_commits {
				// Lookup the reference
				if let Ok(referenced_commit) = index.lookup_svn_revision(*svn_revision) {
					forward_references
						.entry(commit)
						.and_modify(|referenced_commits| referenced_commits.push(referenced_commit))
						.or_insert_with(|| vec![referenced_commit]);
					backward_references
						.entry(referenced_commit)
						.and_modify(|referencing_commits| referencing_commits.push(commit))
						.or_insert_with(|| vec![commit]);
				} else {
					#[cfg(debug_assertions)]
					{
						eprintln!(
							"[WARNING] SVN revision `{svn_revision}` referenced by commit `{}` \
							 could not be found.",
							commit.git_revision
						);
					}
				}
			}
		}
		index.forward_references = forward_references;
		index.backward_references = backward_references;

		// Return the completed index
		Ok(index)
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

	pub fn get_commit_forward_references(&self, commit: &'a Commit) -> Vec<&'a Commit> {
		self.forward_references
			.get(commit)
			.map_or(Vec::new(), Clone::clone)
	}

	pub fn get_commit_backward_references(&self, commit: &'a Commit) -> Vec<&'a Commit> {
		self.backward_references
			.get(commit)
			.map_or(Vec::new(), Clone::clone)
	}
}

fn is_likely_a_real_git_revision(potential_git_revision: &str) -> bool {
	const ASCII_HEX_ALPHA_CHARS: &[char] =
		&['a', 'b', 'c', 'd', 'e', 'f', 'A', 'B', 'C', 'D', 'E', 'F'];

	/// Checks if a string is the same character repeated over and over.
	fn is_repeated_char(s: &str) -> bool {
		if s.is_empty() {
			return false;
		}

		let first_char = s.chars().next().unwrap();

		s.trim_end_matches(first_char).is_empty()
	}

	potential_git_revision.contains(ASCII_HEX_ALPHA_CHARS)
		&& !is_repeated_char(potential_git_revision)
}

//! The module providing constants that need to be shared between several
//! modules.

// Constants
pub const SHA1_HASH_LENGTH: usize = 20;
pub const SHA1_HASH_ASCII_LENGTH: usize = SHA1_HASH_LENGTH * 2;
/// This value comes from a Git SVN migration, and prefixes the data about the
/// original corresponding SVN commit.
///
/// https://github.com/git/git/blob/master/git-svn.perl
pub const GIT_SVN_ID_STR: &str = "git-svn-id";

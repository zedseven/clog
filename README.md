# build-revmap
[![License: MIT/Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

A simple utility for building a revision map for a Git repo that was converted from an SVN
repository by `git-svn` with the `--metadata` flag supplied.

This tool relies on the line that `git-svn` adds to the end of every commit message, with the SVN URL and revision number.

It can output the revision map in Markdown format, or in binary format
([similar to what `git-svn` itself uses](https://github.com/git/git/blob/eea7033409a0ed713c78437fc76486983d211e25/perl/Git/SVN.pm#L2188-L2214)).

## Project License
This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  http://opensource.org/licenses/MIT)

at your option.

### Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in *build-revmap* by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.

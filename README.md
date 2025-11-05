<img src="logo.png" alt="Logo" title="Logo" align="right" width="20%">

# CLog

[![License: MIT/Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

A utility for pulling information from the Git commit log of a repo, then processing it into actionable data.
The name comes from the phrase "Commit Log", but you can also think of it as the variety of shoe.

It works by calling `git log` internally then processing the data, following referenced commit revisions, collecting
Jira tickets, etc., then displaying the information in a useful way.

This tool relies on the line that `git-svn` adds to the end of every commit message when its `--metadata` flag is
provided, which contains the SVN URL and revision number.

It can also generate an SVN to Git revision map from this information, in Markdown or in binary format
([similar to what
`git-svn` itself uses](https://github.com/git/git/blob/eea7033409a0ed713c78437fc76486983d211e25/perl/Git/SVN.pm#L2188-L2214)).

Please note that this tool is somewhat specialised, and may not be useful out of the box for other use-cases.
It's mainly been designed around the Atlassian ecosystem (expecting ticket names to be formatted like Jira does,
for example).
Feel free to [open an issue](https://github.com/zedseven/clog/issues/new) or submit a PR if you have something to add.

## Project License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in *clog* by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.

## Attribution

The logo is [used with permission](logo-certificate.pdf)
from [Flaticon](https://www.flaticon.com/free-icon/clog_1076256),
and made by Freepik.

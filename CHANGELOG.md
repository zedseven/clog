# Changelog

All notable changes to this project will be documented in this file.

## [0.9.1] - 2025-11-07

### Features

- Change the automatic upstream functionality to match up branch names with their remotes instead of trying to use Git's upstream syntax. ([39e175da](https://github.com/zedseven/clog/commit/39e175da))
- Change the automatic upstream functionality to use a combination of Git's upstream syntax *and* matching up branch names with their remotes. ([f0210bca](https://github.com/zedseven/clog/commit/f0210bca))

## [0.9.0] - 2025-11-06

### Features

- Add functionality that is on by default to automatically use the upstream remote variants of branches instead of local ones. ([7dcade92](https://github.com/zedseven/clog/commit/7dcade92))
- Add functionality that is on by default to automatically run `git fetch` before collecting data. ([b3180e77](https://github.com/zedseven/clog/commit/b3180e77))
- Document the fact that the `ticket-prefix` option can also be used to turn tickets into full URLs. ([7725606d](https://github.com/zedseven/clog/commit/7725606d))
- Print a message notifying the user that the output has been copied to the clipboard even when the OS doesn't require the program to remain running for the clipboard contents to be valid. ([8079df4a](https://github.com/zedseven/clog/commit/8079df4a))
- Extend the list of operating systems known to not persist clipboard contents to include BSD and Solaris OSes. ([0ca25003](https://github.com/zedseven/clog/commit/0ca25003))
- Print the clipboard notification regardless of the OS, then print the additional information about clipboard persistence if necessary. ([959dc5f3](https://github.com/zedseven/clog/commit/959dc5f3))
- Change the way important informational messages are highlighted during display, to use `[NOTE]` and `[WARNING]`. ([6298a66a](https://github.com/zedseven/clog/commit/6298a66a))

## [0.8.3] - 2025-11-05

### Continuous Integration

- Only compile artifacts when new tags are pushed. ([c43f2c7c](https://github.com/zedseven/clog/commit/c43f2c7c))

### Miscellaneous Tasks

- Add a logo. ([3e2e0e03](https://github.com/zedseven/clog/commit/3e2e0e03))
- Set up executable packaging for Windows so that the executable has its icon set to the new logo. ([26be3c95](https://github.com/zedseven/clog/commit/26be3c95))
- Update the copyright year in `LICENSE-MIT`. ([ebfe9964](https://github.com/zedseven/clog/commit/ebfe9964))
- Add additional Windows resource properties to `Cargo.toml`. ([dd2c3ed9](https://github.com/zedseven/clog/commit/dd2c3ed9))

## [0.8.2] - 2025-10-30

### Features

- Trim the chopped summary string before adding the ellipsis, to avoid awkward whitespace. ([bf9fed1b](https://github.com/zedseven/clog/commit/bf9fed1b))

## [0.8.1] - 2025-10-30

### Features

- Chop the commit summaries to a limited number of characters. ([835cbd9b](https://github.com/zedseven/clog/commit/835cbd9b))

### Miscellaneous Tasks

- Remove `lazy_static` as a dependency. ([c68b8108](https://github.com/zedseven/clog/commit/c68b8108))

## [0.8.0] - 2025-10-30

### Bug Fixes

- Add `--reflog` as an option to `git log` when collecting all commits from the repository, to gain visibility of commits that have become orphaned. ([7f4fb102](https://github.com/zedseven/clog/commit/7f4fb102))
- Prevent revert commits from being treated like cherry-picks. ([e02a4036](https://github.com/zedseven/clog/commit/e02a4036))
- Make the `simple-ticket-list` option conflict with the `show-commits` option. ([2e593f85](https://github.com/zedseven/clog/commit/2e593f85))
- Fix a small issue with the simple ticket list display that could occur if there are commits with no Jira ticket. ([bfad7d92](https://github.com/zedseven/clog/commit/bfad7d92))

### Continuous Integration

- Update the `upload-artifact` action to version 4. ([158bafc3](https://github.com/zedseven/clog/commit/158bafc3))

### Documentation

- Update the project description slightly. ([0147b4d9](https://github.com/zedseven/clog/commit/0147b4d9))
- Update the crate description in `main.rs` to match what is in `Cargo.toml`. ([ae26686f](https://github.com/zedseven/clog/commit/ae26686f))
- Add a note to `README.md` that the tool is somewhat specialised for the Atlassian ecosystem. ([1dd2455c](https://github.com/zedseven/clog/commit/1dd2455c))

### Features

- Add a new option, `simple-ticket-list`, that causes the ticket lists to be output in a simple, comma-separated format that is perfect for pasting into Confluence's "Jira" macro. ([578bd644](https://github.com/zedseven/clog/commit/578bd644))
- Identify merges, cherry-picks, and reverts as separate commit types in the display. ([69deb6be](https://github.com/zedseven/clog/commit/69deb6be))
- Show commit summaries when commit information is enabled. ([45591a6f](https://github.com/zedseven/clog/commit/45591a6f))
- Display the count of commits alongside the count of Jira tickets for each list. ([a4e4113c](https://github.com/zedseven/clog/commit/a4e4113c))

### Miscellaneous Tasks

- Update the Rust toolchain version to `nightly-2025-10-01`. ([637baded](https://github.com/zedseven/clog/commit/637baded))
- Update `.gitignore` to add an exclusion for the testing repository directory. ([c0dac50f](https://github.com/zedseven/clog/commit/c0dac50f))

### Refactor

- Apply Clippy lints that were suggested after updating the Rust toolchain version. ([1c638b5c](https://github.com/zedseven/clog/commit/1c638b5c))

## [0.7.1] - 2024-02-08

### Bug Fixes

- Fix a stupid copy-paste issue that affects the way commit references are collected. ([069f1464](https://github.com/zedseven/clog/commit/069f1464))

### Continuous Integration

- Change the generated artifacts' names to be more descriptive and to be closer to the release artifact names. ([ff2c0fc1](https://github.com/zedseven/clog/commit/ff2c0fc1))

### Features

- Add a new option, `local-branches`, that searches local branches instead of remote ones as part of the `search` functionality. ([3b3877fa](https://github.com/zedseven/clog/commit/3b3877fa))

## [0.7.0] - 2024-02-07

### Bug Fixes

- Only show the `Branches` and `Tags` sections if they actually have values to display. ([464a22ac](https://github.com/zedseven/clog/commit/464a22ac))
- Adjust the heuristic for detecting merge commits to only recognise messages that mention merging if they also reference at least one other commit. ([9dd9167a](https://github.com/zedseven/clog/commit/9dd9167a))

### Features

- Add a new subcommand, `search`, that searches for all branches where a set of tickets have changes. ([155d6fef](https://github.com/zedseven/clog/commit/155d6fef))
- Add a new option, `search-tags`, that also searches tags as part of the `search` functionality. ([6cedfe2c](https://github.com/zedseven/clog/commit/6cedfe2c))
- Only show the warnings about missing revisions in debug mode, and filter a bit before displaying them. ([b20bd9da](https://github.com/zedseven/clog/commit/b20bd9da))
- Add `repo-dir` as an alias for the `repo` option. ([a92f3a60](https://github.com/zedseven/clog/commit/a92f3a60))

### Miscellaneous Tasks

- Make `release.sh` output the commands to run to push the changes to the remote. ([0e36454d](https://github.com/zedseven/clog/commit/0e36454d))
- Pin the Rust toolchain version and set up the project with `direnv` & Nix. ([cb35e9d5](https://github.com/zedseven/clog/commit/cb35e9d5))
- Update the Rust toolchain version to `nightly-2023-12-05`. ([bece85a7](https://github.com/zedseven/clog/commit/bece85a7))
- Update `flake.nix` to have build support. ([f65e7bb4](https://github.com/zedseven/clog/commit/f65e7bb4))
- Move `toolchain.toml` to `rust-toolchain.toml`. ([6e3f09cc](https://github.com/zedseven/clog/commit/6e3f09cc))
- Add `cargo` as a component to `rust-toolchain.toml`. ([365df8dd](https://github.com/zedseven/clog/commit/365df8dd))
- Update `cliff.toml` to use a regular expression for `tag_pattern`. This is required due to orhun/git-cliff#318. ([aa4a16d6](https://github.com/zedseven/clog/commit/aa4a16d6))
- Update `cliff.toml` to only use the first line of each commit message. ([2f982716](https://github.com/zedseven/clog/commit/2f982716))

## [0.6.0] - 2023-10-26

### Features

- Add a new option, `copy-to-clipboard`, that copies the output to the clipboard automatically. This makes it easy to paste elsewhere with the correct formatting. ([9ed11d90](https://github.com/zedseven/clog/commit/9ed11d90))
- Move the `-m` short alias from `include-merge-commits` to `include-mentioned`, and add `-M` as a short alias for `include-merge-commits`. ([90e70ce7](https://github.com/zedseven/clog/commit/90e70ce7))
- Add `-P` as a short alias for `ticket-prefix`. ([a21b080f](https://github.com/zedseven/clog/commit/a21b080f))

### Miscellaneous Tasks

- Refactor `release.sh` and change the `git-cliff` command used to generate release notes for a tag. ([3ab666bb](https://github.com/zedseven/clog/commit/3ab666bb))

## [0.5.0] - 2023-10-06

### Bug Fixes

- Do not show the commit hashes for commits without Jira tickets anymore. It adds too much noise. ([21826e60](https://github.com/zedseven/clog/commit/21826e60))
- Change the text `<No Jira Ticket>` to `*No Jira Ticket*`, to be more Markdown-friendly. ([984bb5a9](https://github.com/zedseven/clog/commit/984bb5a9))

### Continuous Integration

- Add CI. ([705e2d4c](https://github.com/zedseven/clog/commit/705e2d4c))
- Fix a mistake where the completed artifacts still use the wrong name. ([618fc22f](https://github.com/zedseven/clog/commit/618fc22f))

### Features

- Add a new subcommand, `compare`, that compares two branches to quickly tell what their differences are. ([7c858541](https://github.com/zedseven/clog/commit/7c858541))
- Remove the unhelpful `flatten` option, and replace it with a new `show-commits` option *that is off by default*. Commit hashes will now not be shown unless requested, since they add a lot of noise for little benefit. ([d5ad8ace](https://github.com/zedseven/clog/commit/d5ad8ace))
- Display commits without Jira tickets in the list. ([afb974cf](https://github.com/zedseven/clog/commit/afb974cf))
- Add a new option, `ticket-prefix`, that optionally adds a user-defined prefix to the start of each ticket in the output. This is makes the output more directly-usable with external tools, like turning each ticket into a tag in Obsidian. ([3e93cc9f](https://github.com/zedseven/clog/commit/3e93cc9f))
- Filter out cherry-picks and SVN merges that are on both objects. This behaviour can be disabled with the new option, `include-cherry-picks`. ([edcf92bc](https://github.com/zedseven/clog/commit/edcf92bc))
- Rename the option `include-merges` to `include-merge-commits` to avoid ambiguity with the new option `include-cherry-picks`. Note that an alias still exists with the old name. ([610491bb](https://github.com/zedseven/clog/commit/610491bb))
- Display a marker next to commits that are likely to be merges. ([0dd5fed0](https://github.com/zedseven/clog/commit/0dd5fed0))
- Display commit revisions in backticks, to make the output more Markdown-friendly. ([ef6dcc02](https://github.com/zedseven/clog/commit/ef6dcc02))

### Miscellaneous Tasks

- Set up the repository for automated changelog & tag generation using git-cliff. ([c64eee71](https://github.com/zedseven/clog/commit/c64eee71))

## [0.4.0] - 2023-01-31

### Bug Fixes

- Fix unwelcome behaviour where top-level commits would be first processed as referenced commits, and as a result, their Jira tickets didn't appear in the list. ([ea055828](https://github.com/zedseven/clog/commit/ea055828))

### Documentation

- Update crate description. ([5e30de75](https://github.com/zedseven/clog/commit/5e30de75))
- Update the description for `revspec`. ([c5cf0ffc](https://github.com/zedseven/clog/commit/c5cf0ffc))
- Fix the CLI help display. ([be9cf9f5](https://github.com/zedseven/clog/commit/be9cf9f5))
- Update `README.md`. ([7f1e60af](https://github.com/zedseven/clog/commit/7f1e60af))
- Update crate homepage. ([db3f40b8](https://github.com/zedseven/clog/commit/db3f40b8))

### Features

- Initial implementation of `clog`. ([d0c5c521](https://github.com/zedseven/clog/commit/d0c5c521))
- Provide usable results for the `list` subcommand. ([29406046](https://github.com/zedseven/clog/commit/29406046))
- Develop the results' display further. ([bf50450b](https://github.com/zedseven/clog/commit/bf50450b))
- Improve the behaviour of `--filepath`. ([ae9b9bbb](https://github.com/zedseven/clog/commit/ae9b9bbb))
- Improve the Jira ticket detection. ([5536ca92](https://github.com/zedseven/clog/commit/5536ca92))
- Re-implement the `build-revmap` functionality in the `revmap` subcommand. ([5c326eef](https://github.com/zedseven/clog/commit/5c326eef))
- Implement commit ordering. ([c776a46a](https://github.com/zedseven/clog/commit/c776a46a))
- Remove merge commits by default. ([2152ac9d](https://github.com/zedseven/clog/commit/2152ac9d))
- Display the affected filepaths in use. ([0f342939](https://github.com/zedseven/clog/commit/0f342939))
- Improve the display to sort the filepaths, wrap SVN revision numbers in backticks, and display the revspec in use. ([07e5c24a](https://github.com/zedseven/clog/commit/07e5c24a))
- Allow spaces in the revspec. ([8f161816](https://github.com/zedseven/clog/commit/8f161816))
- Fix the ordering of referenced commits. ([f028b282](https://github.com/zedseven/clog/commit/f028b282))
- Make `revspec` into a positional argument. ([31ec95bd](https://github.com/zedseven/clog/commit/31ec95bd))
- Add short aliases for the CLI. ([118b679c](https://github.com/zedseven/clog/commit/118b679c))

### Miscellaneous Tasks

- Update dependencies. ([515c54d5](https://github.com/zedseven/clog/commit/515c54d5))
- Fix the bad text in `LICENSE-MIT`. ([cab85887](https://github.com/zedseven/clog/commit/cab85887))

### Refactor

- Remove unused imports. ([63113c96](https://github.com/zedseven/clog/commit/63113c96))
- Clean up old modules. ([819a8fa7](https://github.com/zedseven/clog/commit/819a8fa7))

### Optimisation

- Remove the Jira ticket collection from the search operation, so it can be done only when requested. ([51b97c2f](https://github.com/zedseven/clog/commit/51b97c2f))

## [0.3.0] - 2023-01-31

### Features

- Begin the overhaul that converts `build-revmap` into `clog`. ([e7cbe299](https://github.com/zedseven/clog/commit/e7cbe299))

## [0.2.0] - 2023-01-31

### Features

- Add a new option, `--markdown-basic`. ([7f745736](https://github.com/zedseven/clog/commit/7f745736))

## [0.1.0] - 2023-01-31

### Features

- Initial implementation. ([4b5cac0e](https://github.com/zedseven/clog/commit/4b5cac0e))

<!-- generated by git-cliff -->

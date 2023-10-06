# Changelog

All notable changes to this project will be documented in this file.

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

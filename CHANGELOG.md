# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses
[Semantic Versioning](https://semver.org/).

## [Unreleased]

The first release, still to be named.

### Added

- Plain-text web tests (`.pv`) and Gherkin `.feature` files that run on real
  Chrome through navigera, in one native binary with no Node.
- `primavera run` (workers, a browser context per scenario, retries,
  `--repeat` and `--shuffle`, traces), `lint` and `list`.
- Writing tests from what was done in a browser: `record`, and
  `from-session` from a navigera session's history.
- `fix`: a failing step's nearest matches on the page, written into the
  file with `--use`.
- `trace` for a recorded run, `skill` (the agent skill), `mcp` (an MCP
  server with navigera's browser tools and primavera's own), and
  `primavera navigera …`, navigera itself, built in.
- A GitHub Action (`action.yml`) and a container image (`Dockerfile`).
- `help` and `--help` print the usage, as running it bare does.

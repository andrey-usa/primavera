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
- `expect request … body` + a comparison: the last matching response's
  body as text, for a response that isn't JSON or a check on its wording.
- `expect js --file <script.js>` [+ a comparison]: a check from a file next
  to the `.pv` (an expression, or a function body that `return`s its
  answer), for one across rows or too long to quote on a line.
- `primavera <command> --help` prints that command's usage and options.
- `print <command>`: a step that shows a read command's result (a value,
  a locator's text, the page's tree) under its scenario in the run's
  output, checking nothing and waiting for nothing.
- `lint` refuses what would fail only in the run: `set NAME = value` under
  a scenario (a file's variable sits at the top; a step `set` is the
  browser's), flags that can't go together (`--text` with `--nth`), and a
  `set` of a setting the browser doesn't have.
- The agent skill (`primavera skill`) leads with the loop that costs an
  agent least: write the scenario from the task, `lint`, run, and fix from
  the failure, which names the closest elements as locators; then `print`,
  one browser session with `--raw` for looking, and `from-session`. It is
  shorter, every example in it runs as shown, and it says nothing of MCP.
- A failure's note that a click changed nothing is left out once a check
  after the click has held (the page did answer it).
- `lint` also refuses a second `base` line (the last one used to win
  silently) and a `js --file` script that isn't there, under `if` and
  `optional` too, where it read as a check that didn't hold. A repeated
  scenario name says where the first is; `use` of a missing file says
  where it is looked for, without `(os error 2)`.
- `if expect text "…"` is the same as `if text "…"`: the word is optional.
- An `if` on `js --file` finds its script next to the `.pv`, as `expect`
  does, from any directory.

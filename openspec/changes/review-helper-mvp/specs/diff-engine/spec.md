## ADDED Requirements

### Requirement: Parse git diff unified format
The system SHALL parse `git diff` unified format stdout into structured `Vec<FileDiff>` containing file paths, status, hunks, and individual lines with line numbers.

#### Scenario: Parse a single modified file diff
- **WHEN** the parser receives git diff stdout for a single modified file
- **THEN** it SHALL return a `FileDiff` with `status: Modified`, correct old/new paths, hunks with line numbers, and categorized `DiffLine` variants (Context/Add/Delete)

#### Scenario: Parse multiple file diffs
- **WHEN** the parser receives git diff stdout with 3 files (1 added, 1 modified, 1 deleted)
- **THEN** it SHALL return 3 `FileDiff` entries with correct `FileStatus` values

#### Scenario: Parse diff with function name in hunk header
- **WHEN** the parser encounters a hunk header like `@@ -42,6 +42,8 @@ fn parse_hunks()`
- **THEN** it SHALL extract `fn parse_hunks()` as the `function_name` field

#### Scenario: Parse diff with no function name
- **WHEN** the parser encounters a hunk header like `@@ -1,3 +1,4 @@`
- **THEN** it SHALL set `function_name` to `None`

#### Scenario: Handle binary file diff
- **WHEN** the parser encounters a diff with "Binary files differ" line
- **THEN** it SHALL set `is_binary: true` and `hunks` to empty

#### Scenario: Handle renamed file
- **WHEN** the parser encounters a diff with `rename from old.rs` and `rename to new.rs`
- **THEN** it SHALL set `status: Renamed` with correct old/new paths

### Requirement: Compute diff statistics
The system SHALL compute additions and deletions count per file from parsed diff lines.

#### Scenario: Count additions and deletions
- **WHEN** a `FileDiff` contains 5 Add lines and 3 Delete lines
- **THEN** `additions` SHALL be 5 and `deletions` SHALL be 3

### Requirement: Read source file for context expansion
The system SHALL read the actual file content from disk to provide context lines beyond what git diff outputs.

#### Scenario: Expand context beyond hunk boundaries
- **WHEN** user requests context expansion for a hunk at lines 42-48 of `src/main.rs`
- **AND** the file has content at lines 35-55
- **THEN** the system SHALL return additional context lines from the file beyond the hunk's default context

#### Scenario: Handle file not found for context expansion
- **WHEN** the source file does not exist (e.g., deleted file in diff)
- **THEN** the system SHALL return no additional context and NOT error

### Requirement: Execute git diff command
The system SHALL execute `git diff` commands via `std::process::Command` and return stdout.

#### Scenario: Diff against HEAD
- **WHEN** the system runs diff in HEAD mode
- **THEN** it SHALL execute `git diff HEAD` and return the stdout

#### Scenario: Diff against main branch
- **WHEN** the system runs diff in main branch mode
- **THEN** it SHALL execute `git merge-base main HEAD` then `git diff <base> HEAD`

#### Scenario: Diff against arbitrary branch
- **WHEN** the system runs diff against branch `feature-x`
- **THEN** it SHALL execute `git fetch origin feature-x` then compute merge-base and diff

#### Scenario: Handle git command failure
- **WHEN** git command returns non-zero exit code
- **THEN** the system SHALL return an error with stderr content

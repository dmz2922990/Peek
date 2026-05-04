## ADDED Requirements

### Requirement: Render diff hunks with syntax highlighting
The system SHALL render each hunk showing line numbers, +/- prefixes, and color-coded lines (green for additions, red for deletions, default for context).

#### Scenario: Render a standard hunk
- **WHEN** a hunk contains context lines, add lines, and delete lines
- **THEN** context lines render with dimmed line numbers, add lines render green with `+` prefix, delete lines render red with `-` prefix

#### Scenario: Render hunk header
- **WHEN** a hunk has a header like `@@ -42,6 +42,8 @@ fn parse()`
- **THEN** the header renders as a highlighted separator line with the function name shown

### Requirement: Scroll through diff content
The system SHALL support vertical scrolling through the diff content with keyboard controls.

#### Scenario: Scroll down one line
- **WHEN** user presses Down arrow or `j`
- **THEN** the diff view scrolls down by one line

#### Scenario: Scroll by page
- **WHEN** user presses Page Down
- **THEN** the diff view scrolls by one viewport height minus overlap

#### Scenario: Jump to first/last hunk
- **WHEN** user presses `g`
- **THEN** the diff view scrolls to the first hunk of the first file
- **WHEN** user presses `G`
- **THEN** the diff view scrolls to the last hunk of the last file

### Requirement: Expand and collapse context lines
The system SHALL allow users to expand or collapse context lines around changes.

#### Scenario: Expand context around current hunk
- **WHEN** user presses `+` while viewing a hunk
- **THEN** additional surrounding lines from the source file are shown (read from disk)

#### Scenario: Collapse context
- **WHEN** user presses `-` while viewing expanded context
- **THEN** context lines are collapsed back to the default count

#### Scenario: Context expansion respects file boundaries
- **WHEN** user expands context at the beginning of a file (line 1)
- **THEN** no lines above line 1 are shown

### Requirement: Switch diff target
The system SHALL support switching between HEAD, main branch, and arbitrary branch as the diff comparison target.

#### Scenario: Switch to main branch diff
- **WHEN** user presses Tab and selects "main" from the diff target selector
- **THEN** the diff view reloads with `git diff <merge-base-of-main-and-HEAD> HEAD`

#### Scenario: Switch to HEAD diff
- **WHEN** user selects "HEAD" from the diff target selector
- **THEN** the diff view reloads with `git diff HEAD`

### Requirement: Virtualized rendering for large diffs
The system SHALL only render visible lines to maintain performance with large diffs.

#### Scenario: Handle diff with 10,000+ lines
- **WHEN** the diff contains more than 10,000 lines
- **THEN** only the lines within the current viewport are rendered
- **AND** scrolling remains responsive

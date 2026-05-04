## ADDED Requirements

### Requirement: Render file list from diff data
The system SHALL render a scrollable list of changed files in the left panel, grouped by directory structure.

#### Scenario: Display file list
- **WHEN** diff data contains 3 files: `src/main.rs`, `src/diff/parser.rs`, `README.md`
- **THEN** the file tree displays directories `src/` and `.` with files nested under them

#### Scenario: Show file change indicators
- **WHEN** a file in the list has `status: Added`
- **THEN** the file entry SHALL show a green `+` indicator
- **WHEN** a file has `status: Deleted`
- **THEN** the file entry SHALL show a red `-` indicator
- **WHEN** a file has `status: Modified`
- **THEN** the file entry SHALL show a yellow `~` indicator

### Requirement: Keyboard navigation in file tree
The system SHALL support navigating the file list with up/down arrow keys and Enter to select a file.

#### Scenario: Move selection up and down
- **WHEN** user presses Down arrow
- **THEN** selection moves to the next file in the list
- **WHEN** user presses Up arrow at the top
- **THEN** selection stays at the top

#### Scenario: Select file to display in diff view
- **WHEN** user presses Enter on a file entry
- **THEN** the diff view scrolls to and displays the diff for that file
- **AND** focus shifts to the diff view

### Requirement: Toggle file tree visibility
The system SHALL support a configurable keyboard shortcut to toggle the file tree panel open and closed.

#### Scenario: Toggle file tree off
- **WHEN** user presses the configured toggle key (default Ctrl+T)
- **THEN** the file tree panel collapses and diff view takes full width

#### Scenario: Toggle file tree on
- **WHEN** user presses the toggle key again
- **THEN** the file tree panel reappears with previous selection preserved

### Requirement: Show file-level diff statistics
The system SHALL show additions and deletions count next to each file entry.

#### Scenario: Display per-file stats
- **WHEN** file `parser.rs` has 5 additions and 3 deletions
- **THEN** the file tree entry shows `+5 -3` next to the filename

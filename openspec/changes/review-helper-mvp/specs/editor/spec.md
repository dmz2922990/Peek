## ADDED Requirements

### Requirement: Open file in embedded editor
The system SHALL open the current file in an L2 embedded editor when user presses `e` in the diff view.

#### Scenario: Open editor for a file
- **WHEN** user presses `e` while viewing a diff
- **THEN** the current file opens in an embedded editor with the cursor at the current diff line
- **AND** the editor takes over the main view area

#### Scenario: Editor shows file content
- **WHEN** the editor opens for `src/parser.rs`
- **THEN** the full file content is displayed and editable

### Requirement: Basic editing operations
The system SHALL support cursor movement (arrows, Home/End, Page Up/Down), character insert, delete (Backspace/Delete), and multi-line editing.

#### Scenario: Edit and save
- **WHEN** user modifies text and presses Ctrl+S
- **THEN** the file is saved to disk
- **AND** a confirmation indicator appears

#### Scenario: Exit editor without saving
- **WHEN** user presses Escape
- **THEN** if there are unsaved changes, a confirmation prompt appears
- **AND** if user confirms, the editor closes and returns to diff view
- **AND** the diff view refreshes to reflect any saved changes

### Requirement: Search and replace within editor
The system SHALL support Ctrl+R to open an inline search/replace bar within the editor.

#### Scenario: Find and replace
- **WHEN** user presses Ctrl+R and enters search term and replacement
- **THEN** the first match is highlighted
- **AND** user can replace one-by-one or all at once

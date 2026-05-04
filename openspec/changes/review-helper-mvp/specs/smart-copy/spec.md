## ADDED Requirements

### Requirement: Select diff lines for copying
The system SHALL allow users to select a range of diff lines using visual selection mode.

#### Scenario: Enter visual selection mode
- **WHEN** user presses `v` in the diff view
- **THEN** visual selection mode activates with the current line as the start

#### Scenario: Extend selection
- **WHEN** user moves up/down while in visual selection mode
- **THEN** the selection range extends to include the moved-to lines
- **AND** selected lines are visually highlighted

#### Scenario: Exit visual selection without copying
- **WHEN** user presses Escape in visual selection mode
- **THEN** selection is cleared and mode returns to normal

### Requirement: Copy with Markdown enrichment
The system SHALL copy the selected diff content to the system clipboard with a Markdown header containing file path, line range, and function name.

#### Scenario: Copy selected diff lines
- **WHEN** user presses `y` with a selection active
- **THEN** the clipboard contains:
  1. A header line: `📄 <file_path>:<start_line>-<end_line> → <function_name>`
  2. An empty line
  3. A fenced diff code block with the selected diff lines

#### Scenario: Copy with no function name available
- **WHEN** the diff hunk has no function name in its header
- **THEN** the header line omits the `→ <function_name>` part

#### Scenario: Copy spans multiple hunks
- **WHEN** the selection spans multiple hunks in the same file
- **THEN** all selected lines are included in the diff code block
- **AND** the header shows the full line range

### Requirement: Copy single line
The system SHALL support copying the current line without explicit selection via `yy`.

#### Scenario: Yank current line
- **WHEN** user presses `yy` in normal mode
- **THEN** the current diff line is copied to clipboard with the Markdown header

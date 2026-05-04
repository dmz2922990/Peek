## ADDED Requirements

### Requirement: Preserve scroll position across diff refreshes
The system SHALL track the scroll position using a stable anchor and restore it when diff content is refreshed.

#### Scenario: Preserve position after git commit
- **WHEN** user commits changes and the diff refreshes
- **THEN** the diff view scrolls to the same logical position (same file and hunk) as before the commit

#### Scenario: Preserve position after context expansion
- **WHEN** user expands context lines and the diff content changes
- **THEN** the cursor stays on the same logical line

### Requirement: Use stable line anchors
The system SHALL use file path + hunk header + line offset as a stable anchor to track position, not raw line numbers.

#### Scenario: Anchor survives line insertion
- **WHEN** a diff refresh adds lines above the current position
- **THEN** the anchor (file + hunk + offset) resolves to the correct new position
- **AND** the same logical content is visible

#### Scenario: Anchor points to removed content
- **WHEN** a diff refresh removes the hunk the anchor points to
- **THEN** the system scrolls to the nearest remaining hunk in the same file

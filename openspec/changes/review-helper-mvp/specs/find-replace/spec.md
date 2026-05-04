## ADDED Requirements

### Requirement: Open find bar in diff view
The system SHALL provide a search bar that appears at the bottom of the diff view when user presses `/`.

#### Scenario: Activate find bar
- **WHEN** user presses `/` in the diff view
- **THEN** a search input bar appears at the bottom
- **AND** keyboard focus moves to the search input

#### Scenario: Close find bar
- **WHEN** user presses Escape in the find bar
- **THEN** the find bar closes and focus returns to the diff view

### Requirement: Search within diff content
The system SHALL search for the entered text within all visible diff lines and highlight matches.

#### Scenario: Find matches
- **WHEN** user types a search term and presses Enter
- **THEN** all matching lines in the diff are highlighted
- **AND** the view scrolls to the first match

#### Scenario: No matches found
- **WHEN** user searches for a term with no matches
- **THEN** the status bar shows "No matches found"

### Requirement: Navigate between search results
The system SHALL support jumping between search matches with `n` (next) and `N` (previous).

#### Scenario: Jump to next match
- **WHEN** user presses `n` after searching
- **THEN** the diff view scrolls to the next match, wrapping to the first if at the end

#### Scenario: Jump to previous match
- **WHEN** user presses `N` after searching
- **THEN** the diff view scrolls to the previous match, wrapping to the last if at the beginning

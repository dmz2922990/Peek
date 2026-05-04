## ADDED Requirements

### Requirement: Commit dialog
The system SHALL provide a modal commit dialog where users can write a commit message and execute git commit.

#### Scenario: Open commit dialog
- **WHEN** user presses `c`
- **THEN** a centered modal overlay appears with a text area for the commit message
- **AND** shows the current branch name and changed files summary

#### Scenario: Execute commit
- **WHEN** user writes a message and presses Ctrl+Enter
- **THEN** the system executes `git commit -m "<message>"`
- **AND** on success, closes the dialog and refreshes the diff
- **AND** on failure, shows an error toast and keeps the dialog open

#### Scenario: Cancel commit dialog
- **WHEN** user presses Escape in the commit dialog
- **THEN** the dialog closes without executing any git command

### Requirement: Push confirmation
The system SHALL provide a push confirmation dialog showing commits to be pushed.

#### Scenario: Open push dialog
- **WHEN** user presses `p`
- **THEN** a modal shows the count of unpushed commits and the target remote/branch

#### Scenario: Execute push
- **WHEN** user confirms the push
- **THEN** the system executes `git push` (or `git push -u origin <branch>` if no upstream)
- **AND** on success, shows a confirmation toast
- **AND** on failure, shows an error with stderr content

### Requirement: Open browser for PR creation
The system SHALL open the default web browser to the repository's PR creation page.

#### Scenario: Open PR creation URL
- **WHEN** user presses `o`
- **THEN** the system reads the remote URL via `git remote get-url origin`
- **AND** constructs the PR creation URL (e.g., `https://github.com/owner/repo/compare/<branch>`)
- **AND** opens it in the default browser

#### Scenario: No remote configured
- **WHEN** `git remote get-url origin` fails
- **THEN** the system shows an error message "No remote 'origin' configured"

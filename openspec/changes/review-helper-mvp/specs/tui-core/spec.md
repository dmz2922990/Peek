## ADDED Requirements

### Requirement: Terminal initialization and cleanup
The system SHALL initialize crossterm terminal (raw mode, alternate screen) on startup and restore original terminal state on exit.

#### Scenario: Normal startup and shutdown
- **WHEN** user runs `review-helper`
- **THEN** the terminal enters raw mode with alternate screen buffer
- **AND** on exit (via `q` or Ctrl+C), original terminal state is restored

#### Scenario: Panic during execution
- **WHEN** a panic occurs while the TUI is running
- **THEN** the panic handler SHALL restore terminal state before printing the panic message

### Requirement: TEA main event loop
The system SHALL implement a main loop that polls terminal events, converts them to `Message` variants, calls `update()`, then calls `view()` to render.

#### Scenario: Key event flow
- **WHEN** user presses a key
- **THEN** the key is wrapped in `Message::Key(KeyEvent)`, passed to `update()`, and the view is re-rendered

#### Scenario: Terminal resize
- **WHEN** the terminal is resized
- **THEN** `Message::Resize(width, height)` is emitted and the layout adapts

### Requirement: Application state machine
The system SHALL maintain an `AppMode` state that determines which pane is focused and how keyboard input is interpreted.

#### Scenario: Mode transitions
- **WHEN** user is in `DiffViewFocus` mode and presses the file-tree toggle key
- **THEN** mode transitions to `FileTreeFocus`
- **AND** keyboard input is routed to file tree handlers

#### Scenario: Modal overlay for git operations
- **WHEN** user triggers commit in any mode
- **THEN** mode transitions to `GitCommit` and a modal overlay is shown
- **AND** pressing Escape returns to previous mode

### Requirement: Left-right split layout
The system SHALL render a left-right split layout with file tree on the left and diff view on the right, separated by a vertical bar.

#### Scenario: Default layout
- **WHEN** the TUI is rendered in Normal mode
- **THEN** left panel shows file tree (30% width) and right panel shows diff view (70% width)

#### Scenario: Collapsed file tree
- **WHEN** user toggles file tree off
- **THEN** diff view takes full width

### Requirement: Status bar
The system SHALL render a bottom status bar showing diff statistics, current mode, and keybinding hints.

#### Scenario: Status bar content
- **WHEN** the TUI is rendered with a diff loaded
- **THEN** the status bar shows `+N -M files:F` statistics, current diff mode (HEAD/main/branch), and contextual key hints

### Requirement: Async message bridge
The system SHALL use a tokio MPSC channel to deliver async results (git commands, file reads) back to the main TUI loop as `Message` variants.

#### Scenario: Git diff loaded asynchronously
- **WHEN** a background git diff command completes
- **THEN** `Message::DiffLoaded(Vec<FileDiff>)` is sent through the channel
- **AND** the main loop processes it in the next iteration

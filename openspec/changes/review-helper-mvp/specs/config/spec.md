## ADDED Requirements

### Requirement: Load TOML configuration
The system SHALL load configuration from `~/.config/peek/config.toml` on startup, using `serde` + `toml` for parsing.

#### Scenario: Load valid config file
- **WHEN** a valid TOML config file exists at the config path
- **THEN** the system loads keybindings and diff preferences from it

#### Scenario: Config file does not exist
- **WHEN** no config file exists at the config path
- **THEN** the system uses built-in default values and does NOT error

#### Scenario: Config file has invalid TOML
- **WHEN** the config file contains invalid TOML syntax
- **THEN** the system logs a warning and falls back to default values

### Requirement: Configurable keybindings
The system SHALL allow users to remap keyboard shortcuts via the `[keybindings]` section in config.

#### Scenario: Custom keybinding
- **WHEN** config contains `toggle_file_tree = "ctrl+b"`
- **THEN** pressing Ctrl+B toggles the file tree instead of the default Ctrl+T

#### Scenario: Partial keybinding override
- **WHEN** config only overrides `toggle_file_tree` but not other keys
- **THEN** non-overridden keys retain their default bindings

### Requirement: Configurable diff preferences
The system SHALL allow users to configure diff display preferences.

#### Scenario: Custom default context lines
- **WHEN** config contains `default_context_lines = 5`
- **THEN** diff hunks show 5 context lines by default instead of 3

### Requirement: Default configuration values
The system SHALL provide sensible defaults for all configuration options.

#### Scenario: Default keybindings
- **WHEN** no config file exists
- **THEN** the following defaults apply: `q` quit, `j/Down` scroll down, `k/Up` scroll up, `Ctrl+T` toggle file tree, `e` editor, `c` commit, `p` push, `o` open PR, `/` find, `y` copy, `v` visual select, `+/-` context expand/collapse, `Tab` diff target switch

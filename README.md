# sh-rs

This is an implementation of the standard shell in Rust. It is still a work in progress, but it is already usable for basic tasks.

## Implemented

- Print a prompt
- Handle invalid commands
- Implement a REPL
- Implement exit
- Implement echo
- Implement type
- Locate executable files
- Run a program
- Base stages complete!

### Navigation

- The pwd builtin
- The cd builtin: Absolute paths
- The cd builtin: Relative paths
- The cd builtin: Home directory

### Redirection

- Redirect stdout

## Remaining

### Redirection

- Redirect stderr
- Append stdout
- Append stderr 

### Command Completion

- Builtin completion
- Completion with arguments
- Missing completions
- Executable completion
- Multiple completions
- Partial completions 

### Pipelines

- Dual-command pipeline
- Pipelines with built-ins
- Multi-command pipelines

### History

- Listing history
- Limiting history entries
- Up-arrow navigation
- Down-arrow navigation
- Executing commands from history

### History Persistence

- Read history from file
- Write history to file
- Append history to file
- Read history on startup
- Write history on exit
- Append history on exit

### Filename Completion

- File completion
- Nested file completion
- Directory completion
- Missing completions
- Multiple matches
- Partial completions
- Multi-argument completions

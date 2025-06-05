# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust terminal user interface (TUI) application for configuring and running pmars (portable Memory Array Redcode Simulator) commands. Built with the Ratatui framework, it provides an interactive interface for setting pmars command line flags and executing corewar simulations.

## Architecture

- Single-file application structure in `src/main.rs`
- Event-driven architecture using crossterm for terminal input/output
- Main application state held in the `App` struct with pmars configuration
- Two input modes: Normal (navigation) and Editing (field input)
- Form-based UI with configurable fields for all pmars command flags
- Command generation and execution via `std::process::Command`
- Toggle-able output panel for displaying pmars battle results
- Proper terminal initialization with manual setup/cleanup

## Development Commands

- `cargo run` - Build and run the application
- `cargo check` - Check if the code compiles without building
- `cargo build` - Build the application
- `cargo build --release` - Build optimized release version

## Key Dependencies

- `ratatui` - Terminal UI framework for rendering widgets
- `crossterm` - Cross-platform terminal manipulation
- `color-eyre` - Error handling and reporting

## Application Structure

The main `App` struct contains the application state and implements:
- `new()` - Constructor
- `run()` - Main event loop with terminal rendering
- `render()` - UI rendering logic with form layout
- `handle_crossterm_events()` - Terminal event processing
- `on_key_event()` - Keyboard input handling with mode switching
- `generate_pmars_command()` - Creates pmars command string from config
- `run_pmars()` - Executes pmars with configured parameters
- `quit()` - Graceful shutdown

## Key Bindings

### Navigation Mode:
- `j` / `Down Arrow` - Move to next field
- `k` / `Up Arrow` - Move to previous field
- `e` / `Enter` - Edit numeric field or toggle boolean flag
- `Space` - Toggle boolean flags
- `F5` / `Ctrl+r` - Run pmars with current configuration
- `o` - Toggle output panel visibility
- `q` / `Esc` - Quit application

### Editing Mode:
- `Enter` - Save changes and return to navigation
- `Esc` - Cancel changes and return to navigation
- `Backspace` - Delete character
- Type characters to input values

## PMARS Configuration Fields

### Numeric Parameters:
- Rounds (-r): Number of rounds to play (default: 1)
- Core Size (-s): Size of core memory (default: 8000)
- Max Cycles (-c): Maximum cycles per round (default: 80000)
- Max Processes (-p): Maximum processes per warrior (default: 8000)
- Max Length (-l): Maximum warrior length (default: 100)
- Min Distance (-d): Minimum distance between warriors (default: 100)
- P-space Size (-S): Size of P-space (default: 500)
- Display Mode (-v): Display mode number (default: 704)
- Seed Value (-F): Random seed value (optional)

### Boolean Flags:
- Fixed Positioning (-f): Use fixed warrior positioning
- Brief Mode (-b): Suppress warrior listings
- ICWS'88 Mode (-8): Use ICWS'88 standard
- Enter Debugger (-e): Enter debugger after loading

### Default Warrior Files:
- paperone.red
- bomber-basic.red
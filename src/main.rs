use color_eyre::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use std::{io::{self, stdout}, process::Command};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = App::new().run(&mut terminal);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

/// The main application which holds the state and logic of the application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    running: bool,
    /// Current input mode
    input_mode: InputMode,
    /// Current field being edited
    current_field: usize,
    /// pmars configuration
    config: PmarsConfig,
    /// Status message
    status_message: String,
    /// pmars output from last execution
    pmars_output: String,
    /// Whether to show output panel
    show_output: bool,
}

#[derive(Debug, Clone)]
pub enum InputMode {
    Normal,
    Editing,
}

#[derive(Debug, Clone)]
pub struct PmarsConfig {
    pub rounds: String,
    pub core_size: String,
    pub max_cycles: String,
    pub max_processes: String,
    pub max_length: String,
    pub min_distance: String,
    pub pspace_size: String,
    pub display_mode: String,
    pub fixed_positioning: bool,
    pub seed_value: String,
    pub brief_mode: bool,
    pub icws88_mode: bool,
    pub enter_debugger: bool,
    pub warrior_files: Vec<String>,
}

impl Default for PmarsConfig {
    fn default() -> Self {
        Self {
            rounds: "1".to_string(),
            core_size: "8000".to_string(),
            max_cycles: "80000".to_string(),
            max_processes: "8000".to_string(),
            max_length: "100".to_string(),
            min_distance: "100".to_string(),
            pspace_size: "500".to_string(),
            display_mode: "704".to_string(),
            fixed_positioning: false,
            seed_value: "".to_string(),
            brief_mode: false,
            icws88_mode: false,
            enter_debugger: false,
            warrior_files: vec!["paperone.red".to_string(), "bomber-basic.red".to_string()],
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            input_mode: InputMode::Normal,
            current_field: 0,
            config: PmarsConfig::default(),
            status_message: "Press 'e' to edit fields, 'F5' to run pmars, 'o' to toggle output, 'q' to quit".to_string(),
            pmars_output: String::new(),
            show_output: false,
        }
    }
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Run the application's main loop.
    pub fn run(mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        self.running = true;
        while self.running {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_crossterm_events()?;
        }
        Ok(())
    }

    /// Renders the user interface.
    fn render(&mut self, frame: &mut Frame) {
        if self.show_output {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Percentage(40),
                    Constraint::Percentage(40),
                    Constraint::Length(3),
                ])
                .split(frame.area());

            self.render_title(frame, chunks[0]);
            self.render_config_form(frame, chunks[1]);
            self.render_output_panel(frame, chunks[2]);
            self.render_status(frame, chunks[3]);
        } else {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(3),
                ])
                .split(frame.area());

            self.render_title(frame, chunks[0]);
            self.render_config_form(frame, chunks[1]);
            self.render_status(frame, chunks[2]);
        }
    }

    fn render_title(&self, frame: &mut Frame, area: Rect) {
        let title = Paragraph::new("PMARS Configuration Interface")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .block(Block::default().borders(Borders::ALL))
            .wrap(Wrap { trim: true });
        frame.render_widget(title, area);
    }

    fn render_config_form(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        self.render_numeric_fields(frame, chunks[0]);
        self.render_flags_and_files(frame, chunks[1]);
    }

    fn render_numeric_fields(&mut self, frame: &mut Frame, area: Rect) {
        let fields = vec![
            ("Rounds (-r)", &self.config.rounds, 0),
            ("Core Size (-s)", &self.config.core_size, 1),
            ("Max Cycles (-c)", &self.config.max_cycles, 2),
            ("Max Processes (-p)", &self.config.max_processes, 3),
            ("Max Length (-l)", &self.config.max_length, 4),
            ("Min Distance (-d)", &self.config.min_distance, 5),
            ("P-space Size (-S)", &self.config.pspace_size, 6),
            ("Display Mode (-v)", &self.config.display_mode, 7),
            ("Seed Value (-F)", &self.config.seed_value, 8),
        ];

        let items: Vec<ListItem> = fields
            .iter()
            .enumerate()
            .map(|(_i, (label, value, field_idx))| {
                let style = if matches!(self.input_mode, InputMode::Editing) && self.current_field == *field_idx {
                    Style::default().bg(Color::Yellow).fg(Color::Black)
                } else if self.current_field == *field_idx {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                };

                ListItem::new(format!("{}: {}", label, value)).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Numeric Parameters"));
        frame.render_widget(list, area);
    }

    fn render_flags_and_files(&mut self, frame: &mut Frame, area: Rect) {
        let inner_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(8), Constraint::Min(0)])
            .split(area);

        let flags = vec![
            ("Fixed Positioning (-f)", self.config.fixed_positioning, 9),
            ("Brief Mode (-b)", self.config.brief_mode, 10),
            ("ICWS'88 Mode (-8)", self.config.icws88_mode, 11),
            ("Enter Debugger (-e)", self.config.enter_debugger, 12),
        ];

        let flag_items: Vec<ListItem> = flags
            .iter()
            .map(|(label, value, field_idx)| {
                let style = if self.current_field == *field_idx {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                };

                let status = if *value { "ON" } else { "OFF" };
                ListItem::new(format!("{}: {}", label, status)).style(style)
            })
            .collect();

        let flags_list = List::new(flag_items)
            .block(Block::default().borders(Borders::ALL).title("Boolean Flags"));
        frame.render_widget(flags_list, inner_chunks[0]);

        let file_text = format!(
            "Warrior Files:\n{}",
            self.config.warrior_files.join("\n")
        );
        let style = if self.current_field == 13 {
            Style::default().bg(Color::DarkGray)
        } else {
            Style::default()
        };

        let files_widget = Paragraph::new(file_text)
            .style(style)
            .block(Block::default().borders(Borders::ALL).title("Warrior Files"))
            .wrap(Wrap { trim: true });
        frame.render_widget(files_widget, inner_chunks[1]);
    }

    fn render_output_panel(&self, frame: &mut Frame, area: Rect) {
        let output_text = if self.pmars_output.is_empty() {
            "No pmars output yet. Press F5 or Ctrl+r to run pmars.".to_string()
        } else {
            self.pmars_output.clone()
        };

        let output = Paragraph::new(output_text)
            .style(Style::default().fg(Color::White))
            .block(Block::default().borders(Borders::ALL).title("PMARS Battle Output"))
            .wrap(Wrap { trim: true });
        frame.render_widget(output, area);
    }

    fn render_status(&self, frame: &mut Frame, area: Rect) {
        let command = self.generate_pmars_command();
        let text = format!("{}\n\nGenerated command: {}", self.status_message, command);
        
        let status = Paragraph::new(text)
            .style(Style::default().fg(Color::Green))
            .block(Block::default().borders(Borders::ALL).title("Status"))
            .wrap(Wrap { trim: true });
        frame.render_widget(status, area);
    }

    /// Reads the crossterm events and updates the state of [`App`].
    ///
    /// If your application needs to perform work in between handling events, you can use the
    /// [`event::poll`] function to check if there are any events available with a timeout.
    fn handle_crossterm_events(&mut self) -> Result<()> {
        match event::read()? {
            // it's important to check KeyEventKind::Press to avoid handling key release events
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        match self.input_mode {
            InputMode::Normal => self.handle_normal_key(key),
            InputMode::Editing => self.handle_editing_key(key),
        }
    }

    fn handle_normal_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.quit(),
            KeyCode::Char('j') | KeyCode::Down => self.next_field(),
            KeyCode::Char('k') | KeyCode::Up => self.prev_field(),
            KeyCode::Char('e') | KeyCode::Enter => {
                if self.current_field <= 8 {
                    self.input_mode = InputMode::Editing;
                    self.status_message = "Editing mode: type new value, Enter to save, Esc to cancel".to_string();
                } else if self.current_field >= 9 && self.current_field <= 12 {
                    self.toggle_flag();
                }
            }
            KeyCode::Char(' ') => {
                if self.current_field >= 9 && self.current_field <= 12 {
                    self.toggle_flag();
                }
            }
            KeyCode::Char('r') => {
                if KeyModifiers::CONTROL == key.modifiers {
                    self.run_pmars();
                }
            }
            KeyCode::Char('o') => {
                self.show_output = !self.show_output;
                self.status_message = format!("Output panel {}", if self.show_output { "shown" } else { "hidden" });
            }
            KeyCode::F(5) => self.run_pmars(),
            _ => {}
        }
    }

    fn handle_editing_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter => {
                self.input_mode = InputMode::Normal;
                self.status_message = "Press 'e' to edit fields, 'F5' to run pmars, 'o' to toggle output, 'q' to quit".to_string();
            }
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.status_message = "Edit cancelled".to_string();
            }
            KeyCode::Char(c) => {
                self.input_char(c);
            }
            KeyCode::Backspace => {
                self.delete_char();
            }
            _ => {}
        }
    }

    fn next_field(&mut self) {
        self.current_field = (self.current_field + 1) % 14;
    }

    fn prev_field(&mut self) {
        self.current_field = if self.current_field == 0 { 13 } else { self.current_field - 1 };
    }

    fn toggle_flag(&mut self) {
        match self.current_field {
            9 => self.config.fixed_positioning = !self.config.fixed_positioning,
            10 => self.config.brief_mode = !self.config.brief_mode,
            11 => self.config.icws88_mode = !self.config.icws88_mode,
            12 => self.config.enter_debugger = !self.config.enter_debugger,
            _ => {}
        }
    }

    fn input_char(&mut self, c: char) {
        match self.current_field {
            0 => self.config.rounds.push(c),
            1 => self.config.core_size.push(c),
            2 => self.config.max_cycles.push(c),
            3 => self.config.max_processes.push(c),
            4 => self.config.max_length.push(c),
            5 => self.config.min_distance.push(c),
            6 => self.config.pspace_size.push(c),
            7 => self.config.display_mode.push(c),
            8 => self.config.seed_value.push(c),
            _ => {}
        }
    }

    fn delete_char(&mut self) {
        match self.current_field {
            0 => { self.config.rounds.pop(); }
            1 => { self.config.core_size.pop(); }
            2 => { self.config.max_cycles.pop(); }
            3 => { self.config.max_processes.pop(); }
            4 => { self.config.max_length.pop(); }
            5 => { self.config.min_distance.pop(); }
            6 => { self.config.pspace_size.pop(); }
            7 => { self.config.display_mode.pop(); }
            8 => { self.config.seed_value.pop(); }
            _ => {}
        }
    }

    fn generate_pmars_command(&self) -> String {
        let mut cmd = vec!["/usr/local/bin/pmars".to_string()];
        
        if !self.config.rounds.is_empty() && self.config.rounds != "1" {
            cmd.push(format!("-r {}", self.config.rounds));
        }
        if !self.config.core_size.is_empty() && self.config.core_size != "8000" {
            cmd.push(format!("-s {}", self.config.core_size));
        }
        if !self.config.max_cycles.is_empty() && self.config.max_cycles != "80000" {
            cmd.push(format!("-c {}", self.config.max_cycles));
        }
        if !self.config.max_processes.is_empty() && self.config.max_processes != "8000" {
            cmd.push(format!("-p {}", self.config.max_processes));
        }
        if !self.config.max_length.is_empty() && self.config.max_length != "100" {
            cmd.push(format!("-l {}", self.config.max_length));
        }
        if !self.config.min_distance.is_empty() && self.config.min_distance != "100" {
            cmd.push(format!("-d {}", self.config.min_distance));
        }
        if !self.config.pspace_size.is_empty() && self.config.pspace_size != "500" {
            cmd.push(format!("-S {}", self.config.pspace_size));
        }
        if !self.config.display_mode.is_empty() && self.config.display_mode != "704" {
            cmd.push(format!("-v {}", self.config.display_mode));
        }
        if !self.config.seed_value.is_empty() {
            cmd.push(format!("-F {}", self.config.seed_value));
        }
        
        if self.config.fixed_positioning {
            cmd.push("-f".to_string());
        }
        if self.config.brief_mode {
            cmd.push("-b".to_string());
        }
        if self.config.icws88_mode {
            cmd.push("-8".to_string());
        }
        if self.config.enter_debugger {
            cmd.push("-e".to_string());
        }
        
        cmd.extend(self.config.warrior_files.clone());
        
        cmd.join(" ")
    }

    fn run_pmars(&mut self) {
        let mut cmd = Command::new("/usr/local/bin/pmars");
        
        if !self.config.rounds.is_empty() && self.config.rounds != "1" {
            cmd.arg("-r").arg(&self.config.rounds);
        }
        if !self.config.core_size.is_empty() && self.config.core_size != "8000" {
            cmd.arg("-s").arg(&self.config.core_size);
        }
        if !self.config.max_cycles.is_empty() && self.config.max_cycles != "80000" {
            cmd.arg("-c").arg(&self.config.max_cycles);
        }
        if !self.config.max_processes.is_empty() && self.config.max_processes != "8000" {
            cmd.arg("-p").arg(&self.config.max_processes);
        }
        if !self.config.max_length.is_empty() && self.config.max_length != "100" {
            cmd.arg("-l").arg(&self.config.max_length);
        }
        if !self.config.min_distance.is_empty() && self.config.min_distance != "100" {
            cmd.arg("-d").arg(&self.config.min_distance);
        }
        if !self.config.pspace_size.is_empty() && self.config.pspace_size != "500" {
            cmd.arg("-S").arg(&self.config.pspace_size);
        }
        if !self.config.display_mode.is_empty() && self.config.display_mode != "704" {
            cmd.arg("-v").arg(&self.config.display_mode);
        }
        if !self.config.seed_value.is_empty() {
            cmd.arg("-F").arg(&self.config.seed_value);
        }
        
        if self.config.fixed_positioning {
            cmd.arg("-f");
        }
        if self.config.brief_mode {
            cmd.arg("-b");
        }
        if self.config.icws88_mode {
            cmd.arg("-8");
        }
        if self.config.enter_debugger {
            cmd.arg("-e");
        }
        
        for file in &self.config.warrior_files {
            cmd.arg(file);
        }

        match cmd.output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                if output.status.success() {
                    self.pmars_output = stdout.to_string();
                    self.status_message = "pmars executed successfully! Press 'o' to view output.".to_string();
                    self.show_output = true;
                } else {
                    self.pmars_output = format!("ERROR: {}\n\nSTDOUT: {}", stderr, stdout);
                    self.status_message = "pmars failed! Press 'o' to view error details.".to_string();
                    self.show_output = true;
                }
            }
            Err(e) => {
                self.pmars_output = format!("Failed to execute pmars: {}", e);
                self.status_message = "Failed to execute pmars! Press 'o' to view error details.".to_string();
                self.show_output = true;
            }
        }
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}

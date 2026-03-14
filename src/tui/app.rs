use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::backend::Backend;
use ratatui::Terminal;
use std::path::PathBuf;
use std::time::Duration;

use crate::config::{CommandEntry, Config};

/// Which screen the TUI is showing.
#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    CommandList,
    ActionMenu,
    AddCommand,
    EditCommand,
    ConfirmDelete,
}

/// Which field is active in add/edit forms.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputField {
    Tag,
    Command,
    Description,
}

impl InputField {
    pub fn next(self) -> Self {
        match self {
            Self::Tag => Self::Command,
            Self::Command => Self::Description,
            Self::Description => Self::Tag,
        }
    }
}

/// Action menu items.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
    Run,
    Edit,
    Delete,
}

impl Action {
    pub const ALL: [Action; 3] = [Action::Run, Action::Edit, Action::Delete];

    pub fn label(self) -> &'static str {
        match self {
            Self::Run => "Run",
            Self::Edit => "Edit",
            Self::Delete => "Delete",
        }
    }
}

/// TUI application state.
pub struct App {
    pub config: Config,
    pub config_path: PathBuf,
    pub screen: Screen,
    pub should_quit: bool,

    // Command list state
    pub selected_index: usize,
    pub command_tags: Vec<String>,

    // Action menu state
    pub action_index: usize,

    // Input form state
    pub input_tag: String,
    pub input_command: String,
    pub input_desc: String,
    pub active_field: InputField,

    // Pending command to run after TUI exit
    pub pending_command: Option<String>,

    // Status message
    pub status_message: Option<String>,
}

impl App {
    pub fn new(config: Config, config_path: PathBuf) -> Self {
        let command_tags: Vec<String> = config.commands.keys().cloned().collect();
        Self {
            config,
            config_path,
            screen: Screen::CommandList,
            should_quit: false,
            selected_index: 0,
            command_tags,
            action_index: 0,
            input_tag: String::new(),
            input_command: String::new(),
            input_desc: String::new(),
            active_field: InputField::Tag,
            pending_command: None,
            status_message: None,
        }
    }

    /// Refresh the tags list from config.
    fn refresh_tags(&mut self) {
        self.command_tags = self.config.commands.keys().cloned().collect();
        if self.selected_index >= self.command_tags.len() && !self.command_tags.is_empty() {
            self.selected_index = self.command_tags.len() - 1;
        }
    }

    /// Save config to disk.
    fn save_config(&mut self) -> Result<()> {
        self.config.save(&self.config_path)
    }

    /// Get the currently selected tag.
    pub fn selected_tag(&self) -> Option<&str> {
        self.command_tags.get(self.selected_index).map(|s| s.as_str())
    }

    /// Get the currently selected command entry.
    pub fn selected_entry(&self) -> Option<&CommandEntry> {
        self.selected_tag()
            .and_then(|tag| self.config.get(tag))
    }

    /// Main event loop.
    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        while !self.should_quit {
            terminal.draw(|f| super::ui::render(f, self))?;

            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    // Only handle key press events (not release)
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }
                    self.handle_key(key.code)?;
                }
            }
        }
        Ok(())
    }

    /// Dispatch key events based on current screen.
    fn handle_key(&mut self, key: KeyCode) -> Result<()> {
        // Clear status message on any key press
        self.status_message = None;

        match self.screen {
            Screen::CommandList => self.handle_command_list(key)?,
            Screen::ActionMenu => self.handle_action_menu(key)?,
            Screen::AddCommand => self.handle_add_command(key)?,
            Screen::EditCommand => self.handle_edit_command(key)?,
            Screen::ConfirmDelete => self.handle_confirm_delete(key)?,
        }
        Ok(())
    }

    fn handle_command_list(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected_index + 1 < self.command_tags.len() {
                    self.selected_index += 1;
                }
            }
            KeyCode::Enter => {
                if !self.command_tags.is_empty() {
                    self.action_index = 0;
                    self.screen = Screen::ActionMenu;
                }
            }
            KeyCode::Char('a') => {
                self.input_tag.clear();
                self.input_command.clear();
                self.input_desc.clear();
                self.active_field = InputField::Tag;
                self.screen = Screen::AddCommand;
            }
            KeyCode::Char('d') => {
                if !self.command_tags.is_empty() {
                    self.screen = Screen::ConfirmDelete;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_action_menu(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.screen = Screen::CommandList;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.action_index > 0 {
                    self.action_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.action_index + 1 < Action::ALL.len() {
                    self.action_index += 1;
                }
            }
            KeyCode::Enter => {
                let action = Action::ALL[self.action_index];
                match action {
                    Action::Run => {
                        if let Some(entry) = self.selected_entry() {
                            self.pending_command = Some(entry.command.clone());
                            self.should_quit = true;
                        }
                    }
                    Action::Edit => {
                        if let Some(tag) = self.selected_tag() {
                            let tag = tag.to_string();
                            if let Some(entry) = self.config.get(&tag) {
                                self.input_tag = tag;
                                self.input_command = entry.command.clone();
                                self.input_desc = entry.description.clone();
                                self.active_field = InputField::Command;
                                self.screen = Screen::EditCommand;
                            }
                        }
                    }
                    Action::Delete => {
                        self.screen = Screen::ConfirmDelete;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_add_command(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Esc => {
                self.screen = Screen::CommandList;
            }
            KeyCode::Tab | KeyCode::BackTab => {
                self.active_field = self.active_field.next();
            }
            KeyCode::Enter => {
                // Attempt to add the command
                if self.input_tag.is_empty() || self.input_command.is_empty() {
                    self.status_message = Some("Tag and command are required.".to_string());
                    return Ok(());
                }
                match self.config.add(&self.input_tag, &self.input_command, &self.input_desc) {
                    Ok(()) => {
                        self.save_config()?;
                        self.refresh_tags();
                        // Select the newly added tag
                        if let Some(pos) = self.command_tags.iter().position(|t| t == &self.input_tag) {
                            self.selected_index = pos;
                        }
                        self.status_message = Some(format!("Added '{}'", self.input_tag));
                        self.screen = Screen::CommandList;
                    }
                    Err(e) => {
                        self.status_message = Some(e.to_string());
                    }
                }
            }
            KeyCode::Backspace => {
                let field = self.active_input_mut();
                field.pop();
            }
            KeyCode::Char(c) => {
                let field = self.active_input_mut();
                field.push(c);
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_edit_command(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Esc => {
                self.screen = Screen::CommandList;
            }
            KeyCode::Tab | KeyCode::BackTab => {
                // In edit mode, cycle between Command and Description only
                self.active_field = match self.active_field {
                    InputField::Command => InputField::Description,
                    _ => InputField::Command,
                };
            }
            KeyCode::Enter => {
                if self.input_command.is_empty() {
                    self.status_message = Some("Command cannot be empty.".to_string());
                    return Ok(());
                }
                let tag = self.input_tag.clone();
                match self.config.edit(&tag, Some(&self.input_command), Some(&self.input_desc)) {
                    Ok(()) => {
                        self.save_config()?;
                        self.status_message = Some(format!("Updated '{}'", tag));
                        self.screen = Screen::CommandList;
                    }
                    Err(e) => {
                        self.status_message = Some(e.to_string());
                    }
                }
            }
            KeyCode::Backspace => {
                let field = self.active_input_mut();
                field.pop();
            }
            KeyCode::Char(c) => {
                let field = self.active_input_mut();
                field.push(c);
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_confirm_delete(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                if let Some(tag) = self.selected_tag() {
                    let tag = tag.to_string();
                    match self.config.remove(&tag) {
                        Ok(_) => {
                            self.save_config()?;
                            self.refresh_tags();
                            self.status_message = Some(format!("Deleted '{}'", tag));
                        }
                        Err(e) => {
                            self.status_message = Some(e.to_string());
                        }
                    }
                }
                self.screen = Screen::CommandList;
            }
            _ => {
                // Any other key cancels deletion
                self.screen = Screen::CommandList;
            }
        }
        Ok(())
    }

    /// Get mutable reference to the currently active input field.
    fn active_input_mut(&mut self) -> &mut String {
        match self.active_field {
            InputField::Tag => &mut self.input_tag,
            InputField::Command => &mut self.input_command,
            InputField::Description => &mut self.input_desc,
        }
    }
}

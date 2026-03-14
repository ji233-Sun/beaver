use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use super::app::{Action, App, InputField, Screen};

/// Main render dispatcher.
pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),      // main area
            Constraint::Length(3),    // preview
            Constraint::Length(1),    // status / keybindings
        ])
        .split(f.area());

    match app.screen {
        Screen::CommandList => render_command_list(f, app, chunks[0]),
        Screen::ActionMenu => {
            render_command_list(f, app, chunks[0]);
            render_action_menu(f, app);
        }
        Screen::AddCommand => render_input_form(f, app, chunks[0], "Add Command"),
        Screen::EditCommand => render_input_form(f, app, chunks[0], &format!("Edit '{}'", app.input_tag)),
        Screen::ConfirmDelete => {
            render_command_list(f, app, chunks[0]);
            render_confirm_delete(f, app);
        }
    }

    // Preview area
    render_preview(f, app, chunks[1]);

    // Status bar
    render_status_bar(f, app, chunks[2]);
}

fn render_command_list(f: &mut Frame, app: &App, area: Rect) {
    if app.command_tags.is_empty() {
        let msg = Paragraph::new("No commands yet. Press 'a' to add one.")
            .block(Block::default().borders(Borders::ALL).title(" Commands "));
        f.render_widget(msg, area);
        return;
    }

    let items: Vec<ListItem> = app
        .command_tags
        .iter()
        .enumerate()
        .map(|(i, tag)| {
            let desc = app
                .config
                .get(tag)
                .map(|e| {
                    if e.description.is_empty() {
                        e.command.as_str()
                    } else {
                        e.description.as_str()
                    }
                })
                .unwrap_or("");

            let style = if i == app.selected_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let content = Line::from(vec![
                Span::styled(format!("{:<16}", tag), style),
                Span::styled(format!("  {}", desc), Style::default().fg(Color::DarkGray)),
            ]);

            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Commands "))
        .highlight_style(Style::default().bg(Color::DarkGray));

    f.render_widget(list, area);
}

fn render_preview(f: &mut Frame, app: &App, area: Rect) {
    let content = match app.selected_tag() {
        Some(tag) => app
            .config
            .get(tag)
            .map(|e| format!("$ {}", e.command))
            .unwrap_or_default(),
        None => String::new(),
    };

    let preview = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title(" Preview "))
        .wrap(Wrap { trim: false });
    f.render_widget(preview, area);
}

fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let text = if let Some(ref msg) = app.status_message {
        Line::from(Span::styled(msg.as_str(), Style::default().fg(Color::Green)))
    } else {
        let hints = match app.screen {
            Screen::CommandList => "↑↓/jk:Navigate  Enter:Actions  a:Add  d:Delete  q:Quit",
            Screen::ActionMenu => "↑↓/jk:Navigate  Enter:Select  Esc:Back",
            Screen::AddCommand | Screen::EditCommand => "Tab:Next field  Enter:Save  Esc:Cancel",
            Screen::ConfirmDelete => "y:Confirm  any:Cancel",
        };
        Line::from(Span::styled(hints, Style::default().fg(Color::DarkGray)))
    };

    let bar = Paragraph::new(text);
    f.render_widget(bar, area);
}

fn render_action_menu(f: &mut Frame, app: &App) {
    let area = centered_rect(30, 8, f.area());
    f.render_widget(Clear, area);

    let tag_name = app.selected_tag().unwrap_or("?");
    let items: Vec<ListItem> = Action::ALL
        .iter()
        .enumerate()
        .map(|(i, action)| {
            let style = if i == app.action_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let prefix = if i == app.action_index { "> " } else { "  " };
            ListItem::new(Line::from(Span::styled(
                format!("{}{}", prefix, action.label()),
                style,
            )))
        })
        .collect();

    let menu = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" '{}' ", tag_name)),
    );

    f.render_widget(menu, area);
}

fn render_input_form(f: &mut Frame, app: &App, area: Rect, title: &str) {
    let show_tag = app.screen == Screen::AddCommand;

    let constraints = if show_tag {
        vec![
            Constraint::Length(3), // Tag
            Constraint::Length(3), // Command
            Constraint::Length(3), // Description
            Constraint::Min(0),
        ]
    } else {
        vec![
            Constraint::Length(3), // Command
            Constraint::Length(3), // Description
            Constraint::Min(0),
        ]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    let mut idx = 0;

    if show_tag {
        let style = field_style(app.active_field == InputField::Tag);
        let tag_input = Paragraph::new(app.input_tag.as_str())
            .block(Block::default().borders(Borders::ALL).title(" Tag ").border_style(style));
        f.render_widget(tag_input, chunks[idx]);
        idx += 1;
    }

    let cmd_style = field_style(app.active_field == InputField::Command);
    let cmd_input = Paragraph::new(app.input_command.as_str())
        .block(Block::default().borders(Borders::ALL).title(" Command ").border_style(cmd_style));
    f.render_widget(cmd_input, chunks[idx]);
    idx += 1;

    let desc_style = field_style(app.active_field == InputField::Description);
    let desc_input = Paragraph::new(app.input_desc.as_str())
        .block(Block::default().borders(Borders::ALL).title(" Description ").border_style(desc_style));
    f.render_widget(desc_input, chunks[idx]);

    // Form title overlay
    let title_block = Block::default().title(format!(" {} ", title));
    f.render_widget(title_block, area);
}

fn render_confirm_delete(f: &mut Frame, app: &App) {
    let area = centered_rect(40, 5, f.area());
    f.render_widget(Clear, area);

    let tag = app.selected_tag().unwrap_or("?");
    let msg = Paragraph::new(format!("Delete '{}'? (y/N)", tag))
        .block(Block::default().borders(Borders::ALL).title(" Confirm "))
        .style(Style::default().fg(Color::Red));

    f.render_widget(msg, area);
}

/// Style for form fields: highlight active field.
fn field_style(active: bool) -> Style {
    if active {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    }
}

/// Create a centered rectangle of given percentage width and fixed height.
fn centered_rect(percent_x: u16, height: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((area.height.saturating_sub(height)) / 2),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

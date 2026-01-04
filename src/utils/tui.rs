use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;

use crate::api::media::{Media, MediaDetails, Variables, get_media, get_media_details};

pub enum InputMode {
    Normal,
    Editing,
}

pub enum FocusPanel {
    List,
    Details,
}

pub struct App {
    pub username: String,
    pub search_input: String,
    pub input_mode: InputMode,
    pub search_results: Vec<Media>,
    pub results_state: ListState,
    pub selected_media: Option<MediaDetails>,
    pub show_details: bool,
    pub focus_panel: FocusPanel,
    pub details_scroll: u16,
}

impl App {
    pub fn new(username: String) -> App {
        App {
            username,
            search_input: String::new(),
            input_mode: InputMode::Editing,
            search_results: Vec::new(),
            results_state: ListState::default(),
            selected_media: None,
            show_details: false,
            focus_panel: FocusPanel::List,
            details_scroll: 0,
        }
    }

    pub fn scroll_details_up(&mut self) {
        self.details_scroll = self.details_scroll.saturating_sub(1);
    }

    pub fn scroll_details_down(&mut self) {
        self.details_scroll = self.details_scroll.saturating_add(1);
    }

    pub fn switch_focus(&mut self) {
        if self.show_details {
            self.focus_panel = match self.focus_panel {
                FocusPanel::List => FocusPanel::Details,
                FocusPanel::Details => FocusPanel::List,
            };
        }
    }

    pub fn move_selection_up(&mut self) {
        if self.search_results.is_empty() {
            return;
        }

        let i = match self.results_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.search_results.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.results_state.select(Some(i));
    }

    pub fn move_selection_down(&mut self) {
        if self.search_results.is_empty() {
            return;
        }

        let i = match self.results_state.selected() {
            Some(i) => {
                if i >= self.search_results.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.results_state.select(Some(i));
    }

    pub fn get_selected_media(&self) -> Option<&Media> {
        if let Some(i) = self.results_state.selected() {
            self.search_results.get(i)
        } else {
            None
        }
    }
}

pub fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Controls
        ])
        .split(f.area());

    // Header with search bar and username
    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(chunks[0]);

    // Search bar
    let search_text = vec![Line::from(vec![Span::raw(&app.search_input)])];
    let search_paragraph = Paragraph::new(search_text)
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Yellow),
        })
        .block(Block::default().borders(Borders::ALL).title("Search"));
    f.render_widget(search_paragraph, header_chunks[0]);

    // Username display
    let username_text = vec![Line::from(vec![Span::raw(&app.username)])];
    let username_paragraph = Paragraph::new(username_text)
        .style(Style::default().fg(Color::Green))
        .block(Block::default().borders(Borders::ALL).title("User"));
    f.render_widget(username_paragraph, header_chunks[1]);

    // Main content area
    if app.show_details && app.selected_media.is_some() {
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(chunks[1]);

        let list_focused = matches!(app.focus_panel, FocusPanel::List);
        let details_focused = matches!(app.focus_panel, FocusPanel::Details);

        render_results_list(f, app, main_chunks[0], list_focused);
        render_details_panel(f, app, main_chunks[1], details_focused);
    } else {
        render_results_list(f, app, chunks[1], true);
    }

    // Controls footer
    let controls_text = match app.input_mode {
        InputMode::Editing => {
            vec![Line::from(vec![
                Span::styled("Enter", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" Search | "),
                Span::styled("Esc", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" Normal | "),
                Span::styled("↑↓", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" Navigate | "),
                Span::styled("Ctrl+C", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" Quit"),
            ])]
        }
        InputMode::Normal => {
            if app.show_details {
                vec![Line::from(vec![
                    Span::styled("s", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::raw(" Search | "),
                    Span::styled("Tab", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::raw(" Switch Panel | "),
                    Span::styled("↑↓", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::raw(" Navigate/Scroll | "),
                    Span::styled("Esc", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::raw(" Close | "),
                    Span::styled("q/Ctrl+C", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::raw(" Quit"),
                ])]
            } else {
                vec![Line::from(vec![
                    Span::styled("s", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::raw(" Search | "),
                    Span::styled("↑↓", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::raw(" Navigate | "),
                    Span::styled("Enter", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::raw(" Details | "),
                    Span::styled("q/Ctrl+C", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::raw(" Quit"),
                ])]
            }
        }
    };
    let controls_paragraph = Paragraph::new(controls_text)
        .style(Style::default().bg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL).title("Controls"));
    f.render_widget(controls_paragraph, chunks[2]);

    if let InputMode::Editing = app.input_mode {
        f.set_cursor_position((
            header_chunks[0].x + app.search_input.len() as u16 + 1,
            header_chunks[0].y + 1,
        ));
    }
}

fn render_results_list(f: &mut Frame, app: &mut App, area: Rect, focused: bool) {
    let items: Vec<ListItem> = app
        .search_results
        .iter()
        .map(|media| {
            let title = if !media.title.english.is_empty() && media.title.english != "No title" {
                &media.title.english
            } else {
                &media.title.romaji
            };

            ListItem::new(Line::from(vec![Span::raw(title)]))
        })
        .collect();

    let border_style = if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title("Results"),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, &mut app.results_state);
}

fn render_details_panel(f: &mut Frame, app: &App, area: Rect, focused: bool) {
    if let Some(details) = &app.selected_media {
        let mut text = Vec::new();

        let border_style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        text.push(Line::from(vec![
            Span::styled("English: ", Style::default().fg(Color::Cyan)),
            Span::raw(&details.title.english),
        ]));
        text.push(Line::from(vec![
            Span::styled("Romaji: ", Style::default().fg(Color::Cyan)),
            Span::raw(&details.title.romaji),
        ]));
        text.push(Line::from(vec![
            Span::styled("Native: ", Style::default().fg(Color::Cyan)),
            Span::raw(&details.title.native),
        ]));
        text.push(Line::from(""));

        if let Some(media_type) = &details.media_type {
            text.push(Line::from(vec![
                Span::styled("Type: ", Style::default().fg(Color::Cyan)),
                Span::raw(media_type),
            ]));
        }
        if let Some(format) = &details.format {
            text.push(Line::from(vec![
                Span::styled("Format: ", Style::default().fg(Color::Cyan)),
                Span::raw(format),
            ]));
        }
        text.push(Line::from(""));

        if let Some(status) = &details.status {
            text.push(Line::from(vec![
                Span::styled("Status: ", Style::default().fg(Color::Cyan)),
                Span::raw(status),
            ]));
        }
        if let Some(episodes) = details.episodes {
            text.push(Line::from(vec![
                Span::styled("Episodes: ", Style::default().fg(Color::Cyan)),
                Span::raw(episodes.to_string()),
            ]));
        }
        if let Some(chapters) = details.chapters {
            text.push(Line::from(vec![
                Span::styled("Chapters: ", Style::default().fg(Color::Cyan)),
                Span::raw(chapters.to_string()),
            ]));
        }
        text.push(Line::from(""));


        if let Some(score) = details.average_score {
            text.push(Line::from(vec![
                Span::styled("Average Score: ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("{}/100", score),
                    Style::default().fg(Color::Green),
                ),
            ]));
        }
        if let Some(popularity) = details.popularity {
            text.push(Line::from(vec![
                Span::styled("Popularity: ", Style::default().fg(Color::Cyan)),
                Span::raw(popularity.to_string()),
            ]));
        }
        text.push(Line::from(""));

        if let Some(desc) = &details.description {
            text.push(Line::from(vec![Span::styled(
                "Description:",
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            )]));

            // Strip HTML tags from description
            let clean_desc = strip_html_tags(desc);
            text.push(Line::from(clean_desc));
            text.push(Line::from(""));
        }

        if !details.genres.is_empty() {
            text.push(Line::from(vec![
                Span::styled("Genres: ", Style::default().fg(Color::Cyan)),
                Span::raw(details.genres.join(", ")),
            ]));
        }

        let paragraph = Paragraph::new(text)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title("Details"))
            .wrap(Wrap { trim: true })
            .scroll((app.details_scroll, 0));

        f.render_widget(paragraph, area);
    }
}

fn strip_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;

    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ => {
                if !in_tag {
                    result.push(c);
                }
            }
        }
    }

    result
}

pub async fn run_tui(username: String, token: String) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(username);
    let mut last_search = String::new();

    let res = run_app(&mut terminal, &mut app, &mut last_search, &token).await;

    disable_raw_mode()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {:?}", err);
    }

    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    last_search: &mut String,
    token: &str,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                    return Ok(());
                }

                match app.input_mode {
                    InputMode::Editing => match key.code {
                        KeyCode::Enter => {
                            if !app.search_input.is_empty() && app.search_input != *last_search {
                                *last_search = app.search_input.clone();
                                let variables = Variables {
                                    page: 1,
                                    per_page: 20,
                                    search: app.search_input.clone(),
                                };
                                let response = get_media(variables).await;
                                app.search_results = response.data.page.media;
                                if !app.search_results.is_empty() {
                                    app.results_state.select(Some(0));
                                }
                            }
                        }
                        KeyCode::Char(c) => {
                            app.search_input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.search_input.pop();
                        }
                        KeyCode::Esc => {
                            app.input_mode = InputMode::Normal;
                        }
                        KeyCode::Down => {
                            app.input_mode = InputMode::Normal;
                            app.move_selection_down();
                        }
                        KeyCode::Up => {
                            app.input_mode = InputMode::Normal;
                            app.move_selection_up();
                        }
                        _ => {}
                    },
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') => {
                            return Ok(());
                        }
                        KeyCode::Char('s') | KeyCode::Char('i') | KeyCode::Char('/') => {
                            app.input_mode = InputMode::Editing;
                        }
                        KeyCode::Tab => {
                            app.switch_focus();
                        }
                        KeyCode::Down => {
                            match app.focus_panel {
                                FocusPanel::List => {
                                    app.move_selection_down();
                                }
                                FocusPanel::Details => {
                                    app.scroll_details_down();
                                }
                            }
                        }
                        KeyCode::Up => {
                            match app.focus_panel {
                                FocusPanel::List => {
                                    app.move_selection_up();
                                }
                                FocusPanel::Details => {
                                    app.scroll_details_up();
                                }
                            }
                        }
                        KeyCode::Enter => {
                            if let Some(media) = app.get_selected_media() {
                                let media_id = media.id;
                                match get_media_details(media_id, token).await {
                                    Ok(details) => {
                                        app.selected_media = Some(details);
                                        app.show_details = true;
                                        app.focus_panel = FocusPanel::List;
                                        app.details_scroll = 0;
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to load media details: {}", e);
                                    }
                                }
                            }
                        }
                        KeyCode::Esc => {
                            if app.show_details {
                                app.show_details = false;
                                app.focus_panel = FocusPanel::List;
                                app.details_scroll = 0;
                            }
                        }
                        _ => {}
                    },
                }
            }
        }
    }
}

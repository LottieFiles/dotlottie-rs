use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use minifb::{Key, Window, WindowOptions};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, ListState},
    Terminal,
};
use std::{
    io,
    time::{Duration, Instant},
};

const WIDTH: usize = 400;
const HEIGHT: usize = 300;

enum MenuItemType {
    Button { name: String, color: u32 },
    StringInput { name: String, value: String },
    NumberInput { name: String, value: i32 },
    BooleanToggle { name: String, value: bool },
}

struct Menu {
    name: String,
    items: Vec<MenuItemType>,
    state: ListState,
}

impl Menu {
    fn new(name: String, items: Vec<MenuItemType>) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Menu { name, items, state }
    }
}

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut window = Window::new(
        "Color-changing Triangle",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    run_app(&mut terminal, &mut window, &mut buffer)?;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    window: &mut Window,
    buffer: &mut Vec<u32>,
) -> io::Result<()> {
    let mut menus = vec![
        Menu::new(
            "Colors".to_string(),
            vec![
                MenuItemType::Button {
                    name: "Red".to_string(),
                    color: 0xFF0000,
                },
                MenuItemType::Button {
                    name: "Green".to_string(),
                    color: 0x00FF00,
                },
                MenuItemType::Button {
                    name: "Blue".to_string(),
                    color: 0x0000FF,
                },
            ],
        ),
        Menu::new(
            "Inputs".to_string(),
            vec![
                MenuItemType::StringInput {
                    name: "Text".to_string(),
                    value: String::new(),
                },
                MenuItemType::NumberInput {
                    name: "Number".to_string(),
                    value: 0,
                },
                MenuItemType::BooleanToggle {
                    name: "Toggle".to_string(),
                    value: false,
                },
            ],
        ),
        Menu::new(
            "Actions".to_string(),
            vec![
                MenuItemType::Button {
                    name: "Clear".to_string(),
                    color: 0x000000,
                },
                MenuItemType::Button {
                    name: "Random".to_string(),
                    color: 0xFFFFFF,
                },
                MenuItemType::Button {
                    name: "Quit".to_string(),
                    color: 0x000000,
                },
            ],
        ),
    ];

    let mut current_menu = 0;
    let mut selected_color = 0xFF0000;
    let mut last_update = Instant::now();
    let mut input_mode = false;

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Percentage(33),
                        Constraint::Percentage(33),
                        Constraint::Percentage(33),
                    ]
                    .as_ref(),
                )
                .split(f.size());

            for (i, menu) in menus.iter_mut().enumerate() {
                let items: Vec<ListItem> = menu
                    .items
                    .iter()
                    .enumerate()
                    .map(|(j, item)| {
                        let (content, style) = match item {
                            MenuItemType::Button { name, color } => {
                                let style = if i == current_menu
                                    && j == menu.state.selected().unwrap_or(0)
                                {
                                    Style::default()
                                        .fg(Color::Yellow)
                                        .add_modifier(Modifier::BOLD)
                                } else {
                                    Style::default()
                                };
                                (
                                    format!(
                                        "{}{}",
                                        if *color == selected_color { "> " } else { "  " },
                                        name
                                    ),
                                    style,
                                )
                            }
                            MenuItemType::StringInput { name, value } => {
                                let style = if i == current_menu
                                    && j == menu.state.selected().unwrap_or(0)
                                {
                                    if input_mode {
                                        Style::default().fg(Color::Green)
                                    } else {
                                        Style::default().fg(Color::Yellow)
                                    }
                                } else {
                                    Style::default()
                                };
                                (format!("{}: {}", name, value), style)
                            }
                            MenuItemType::NumberInput { name, value } => {
                                let style = if i == current_menu
                                    && j == menu.state.selected().unwrap_or(0)
                                {
                                    if input_mode {
                                        Style::default().fg(Color::Green)
                                    } else {
                                        Style::default().fg(Color::Yellow)
                                    }
                                } else {
                                    Style::default()
                                };
                                (format!("{}: {}", name, value), style)
                            }
                            MenuItemType::BooleanToggle { name, value } => {
                                let style = if i == current_menu
                                    && j == menu.state.selected().unwrap_or(0)
                                {
                                    Style::default().fg(Color::Yellow)
                                } else {
                                    Style::default()
                                };
                                (
                                    format!("{}: {}", name, if *value { "On" } else { "Off" }),
                                    style,
                                )
                            }
                        };
                        ListItem::new(Spans::from(vec![Span::styled(content, style)]))
                    })
                    .collect();

                let items = List::new(items)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(menu.name.clone()),
                    )
                    .highlight_style(Style::default().bg(Color::DarkGray))
                    .highlight_symbol("");

                f.render_stateful_widget(items, chunks[i], &mut menu.state);
            }
        })?;

        if last_update.elapsed() >= Duration::from_millis(16) {
            draw_triangle(buffer, selected_color);
            window.update_with_buffer(buffer, WIDTH, HEIGHT).unwrap();
            last_update = Instant::now();
        }

        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if input_mode {
                    handle_input_mode(&mut menus[current_menu], key);
                    if key.code == KeyCode::Esc {
                        input_mode = false;
                    }
                } else {
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Down => move_selection(&mut menus, &mut current_menu, 1),
                        KeyCode::Up => move_selection(&mut menus, &mut current_menu, -1),
                        KeyCode::Enter => {
                            let menu = &mut menus[current_menu];
                            let i = menu.state.selected().unwrap_or(0);
                            match &mut menu.items[i] {
                                MenuItemType::Button { name, color } => match name.as_str() {
                                    "Quit" => return Ok(()),
                                    "Clear" => selected_color = 0x000000,
                                    "Random" => selected_color = rand::random::<u32>() | 0xFF000000,
                                    _ => selected_color = *color,
                                },
                                MenuItemType::StringInput { .. }
                                | MenuItemType::NumberInput { .. } => {
                                    input_mode = true;
                                }
                                MenuItemType::BooleanToggle { value, .. } => {
                                    *value = !*value;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        if !window.is_open() || window.is_key_down(Key::Escape) {
            return Ok(());
        }
    }
}

fn move_selection(menus: &mut [Menu], current_menu: &mut usize, direction: i32) {
    let menu = &mut menus[*current_menu];
    let mut new_index = menu.state.selected().unwrap_or(0) as i32 + direction;
    if new_index < 0 {
        *current_menu = (*current_menu + menus.len() - 1) % menus.len();
        new_index = menus[*current_menu].items.len() as i32 - 1;
    } else if new_index >= menu.items.len() as i32 {
        *current_menu = (*current_menu + 1) % menus.len();
        new_index = 0;
    }
    menus[*current_menu].state.select(Some(new_index as usize));
}

fn handle_input_mode(menu: &mut Menu, key: event::KeyEvent) {
    let i = menu.state.selected().unwrap_or(0);
    match &mut menu.items[i] {
        MenuItemType::StringInput { value, .. } => match key.code {
            KeyCode::Char(c) => value.push(c),
            KeyCode::Backspace => {
                value.pop();
            }
            _ => {}
        },
        MenuItemType::NumberInput { value, .. } => match key.code {
            KeyCode::Char(c) if c.is_digit(10) => {
                *value = value
                    .saturating_mul(10)
                    .saturating_add(c.to_digit(10).unwrap() as i32);
            }
            KeyCode::Backspace => {
                *value /= 10;
            }
            _ => {}
        },
        _ => {}
    }
}

fn draw_triangle(buffer: &mut Vec<u32>, color: u32) {
    buffer.fill(0);
    let w = WIDTH as i32;
    let h = HEIGHT as i32;
    let vertices = [(w / 2, 50), (w / 4, h - 50), (3 * w / 4, h - 50)];
    for y in 0..h {
        for x in 0..w {
            if point_in_triangle(x, y, vertices) {
                buffer[(y * w + x) as usize] = color;
            }
        }
    }
}

fn point_in_triangle(x: i32, y: i32, vertices: [(i32, i32); 3]) -> bool {
    let [(x1, y1), (x2, y2), (x3, y3)] = vertices;
    let d1 = sign(x, y, x1, y1, x2, y2);
    let d2 = sign(x, y, x2, y2, x3, y3);
    let d3 = sign(x, y, x3, y3, x1, y1);
    let has_neg = (d1 < 0) || (d2 < 0) || (d3 < 0);
    let has_pos = (d1 > 0) || (d2 > 0) || (d3 > 0);
    !(has_neg && has_pos)
}

fn sign(x1: i32, y1: i32, x2: i32, y2: i32, x3: i32, y3: i32) -> i32 {
    (x1 - x3) * (y2 - y3) - (x2 - x3) * (y1 - y3)
}

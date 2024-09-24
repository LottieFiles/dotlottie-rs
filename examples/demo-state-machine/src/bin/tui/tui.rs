use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use dotlottie_rs::{Config, DotLottiePlayer};
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
    fs::{self, File},
    io::{self, Read},
    time::{Duration, Instant},
};

const WIDTH: usize = 400;
const HEIGHT: usize = 300;

/* Timer ---------------------------------------------------------------------------------------- */
struct Timer {
    last_update: Instant,
    prev_frame: f32,
    first: bool,
}

impl Timer {
    fn new() -> Self {
        Self {
            last_update: Instant::now(),
            prev_frame: 0.0,
            first: false,
        }
    }

    fn tick(&mut self, animation: &DotLottiePlayer) {
        let next_frame = animation.request_frame();

        animation.set_frame(next_frame);

        if next_frame != self.prev_frame || !self.first {
            animation.render();
            self.first = true;
        }

        self.last_update = Instant::now(); // Reset the timer
        self.prev_frame = next_frame;
    }
}

/* ----------------------------------------------------------------------------------------- */

struct Button {
    name: String,
    color: u32,
}

fn main() -> Result<(), io::Error> {
    // Set up the terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Set up the Minifb window
    let mut window = Window::new(
        "Color-changing Triangle",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Create a buffer for the triangle
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    /* dotLottie ----------------------------------------------------------------------------------------- */

    let mut lottie_player: DotLottiePlayer = DotLottiePlayer::new(Config {
        background_color: 0xffffffff,
        ..Config::default()
    });

    let mut markers = File::open("./src/bin/tui/pigeon.lottie").expect("no file found");
    let metadatamarkers =
        fs::metadata("./src/bin/tui/pigeon.lottie").expect("unable to read metadata");
    let mut markers_buffer = vec![0; metadatamarkers.len() as usize];
    markers.read(&mut markers_buffer).expect("buffer overflow");

    lottie_player.load_dotlottie_data(&markers_buffer, WIDTH as u32, HEIGHT as u32);

    let mut timer = Timer::new();

    let message: String = fs::read_to_string("./src/bin/tui/no_events_test.json").unwrap();

    let r = lottie_player.load_state_machine_data(&message);

    println!("Load state machine data -> {}", r);

    let s = lottie_player.start_state_machine();

    println!("Start state machine -> {}", s);

    println!("is_playing: {}", lottie_player.is_playing());

    lottie_player.render();

    /* end dotLottie ----------------------------------------------------------------------------------------- */

    // Run the main application loop
    run_app(
        &mut terminal,
        &mut window,
        &mut buffer,
        &mut lottie_player,
        &mut timer,
    )?;

    // Restore terminal
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
    player: &mut DotLottiePlayer,
    timer: &mut Timer,
) -> io::Result<()> {
    let buttons = vec![
        Button {
            name: "Red".to_string(),
            color: 0xFF0000,
        },
        Button {
            name: "Green".to_string(),
            color: 0x00FF00,
        },
        Button {
            name: "Blue".to_string(),
            color: 0x0000FF,
        },
        Button {
            name: "Quit".to_string(),
            color: 0x000000,
        },
    ];
    let mut cursor_position = 0;
    let mut selected_color = buttons[0].color; // Start with the first color
    let mut list_state = ListState::default();
    list_state.select(Some(0));

    let mut last_update = Instant::now();

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                .split(f.size());

            let items: Vec<ListItem> = buttons
                .iter()
                .enumerate()
                .map(|(index, button)| {
                    let style = if index == cursor_position {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };
                    ListItem::new(vec![Spans::from(vec![
                        Span::styled(
                            if button.color == selected_color {
                                "> "
                            } else {
                                "  "
                            },
                            style,
                        ),
                        Span::styled(button.name.clone(), style),
                    ])])
                })
                .collect();

            let items = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Menu"))
                .highlight_style(Style::default().bg(Color::DarkGray))
                .highlight_symbol("");

            f.render_stateful_widget(items, chunks[0], &mut list_state);
        })?;

        // Update the window with a new frame
        // Update and render Minifb window

        if last_update.elapsed() >= Duration::from_millis(16) {
            // draw_triangle(buffer, selected_color);
            // window.update_with_buffer(buffer, WIDTH, HEIGHT).unwrap();
            // last_update = Instant::now();

            timer.tick(&*player);

            let p = &mut *player;

            let (buffer_ptr, buffer_len) = (p.buffer_ptr(), p.buffer_len());

            let buffer = unsafe {
                std::slice::from_raw_parts(buffer_ptr as *const u32, buffer_len as usize)
            };

            window.update_with_buffer(buffer, WIDTH, HEIGHT).unwrap();
            last_update = Instant::now();
        }

        // Handle input
        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Down => {
                        cursor_position = (cursor_position + 1) % buttons.len();
                        list_state.select(Some(cursor_position));
                    }
                    KeyCode::Up => {
                        cursor_position = (cursor_position + buttons.len() - 1) % buttons.len();
                        list_state.select(Some(cursor_position));
                    }
                    KeyCode::Enter => {
                        if cursor_position == buttons.len() - 1 {
                            return Ok(());
                        } else {
                            selected_color = buttons[cursor_position].color;
                        }
                    }
                    _ => {}
                }
            }
        }

        // Check if Minifb window is closed
        if !window.is_open() || window.is_key_down(Key::Escape) {
            return Ok(());
        }
    }
}

fn draw_triangle(buffer: &mut Vec<u32>, color: u32) {
    buffer.fill(0); // Clear the buffer

    let w = WIDTH as i32;
    let h = HEIGHT as i32;

    // Define triangle vertices
    let vertices = [(w / 2, 50), (w / 4, h - 50), (3 * w / 4, h - 50)];

    // Draw the triangle
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

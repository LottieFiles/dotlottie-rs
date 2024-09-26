use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use dotlottie_rs::{
    states::StateTrait, transitions::TransitionTrait, triggers::Trigger, Config, DotLottiePlayer,
};
use minifb::{Key, Window, WindowOptions};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Widget},
    Terminal,
};
use std::{
    fs::{self, File},
    io::{self, Read},
    process,
    sync::mpsc::{self, Receiver, Sender},
    time::{Duration, Instant},
};

const WIDTH: usize = 400;
const HEIGHT: usize = 300;
const LOADED_STATE_MACHINE: &str = "events";

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

enum MenuItemType {
    Button {
        name: String,
        color: u32,
    },
    StringInput {
        name: String,
        value: String,
    },
    NumberInput {
        name: String,
        value: f32,
        buffer: String,
    },
    BooleanToggle {
        name: String,
        value: bool,
    },
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

struct Node {
    x: f64,
    y: f64,
    label: String,
    active: bool,
}

struct Edge {
    from: usize,
    to: usize,
}

struct Graph {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

impl Graph {
    fn new(player: &DotLottiePlayer) -> Self {
        let sm = player.get_state_machine();
        let read_lock = sm.try_read();

        match read_lock {
            Ok(locked_machine) => {
                let optional_machine = &*locked_machine;

                if let Some(machine_engine) = optional_machine {
                    let machine = machine_engine.get_state_machine();

                    let states = machine.states();
                    let mut nodes = Vec::new();
                    let mut edges = Vec::new();

                    for (i, state) in states.iter().enumerate() {
                        let x = (i % 3) as f64 * 0.3 + 0.2;
                        let y = (i / 3) as f64 * 0.3 + 0.2;
                        nodes.push(Node {
                            x,
                            y,
                            label: state.get_name(),
                            active: machine_engine.get_current_state_name() == state.get_name(),
                        });

                        for transition in state.get_transitions() {
                            let target = machine.get_state_by_name(&transition.get_target_state());
                            if let Some(target) = target {
                                let target_index = states
                                    .iter()
                                    .position(|s| s.get_name() == target.get_name())
                                    .unwrap();
                                edges.push(Edge {
                                    from: i,
                                    to: target_index,
                                });
                            }
                        }
                    }

                    return Graph { nodes, edges };
                } else {
                    println!("Error: State machine is None");
                    process::abort();
                }
            }
            Err(err) => {
                println!("Error: {}", err);
                process::abort();
            }
        }
    }

    fn update(&self, player: &DotLottiePlayer) -> Graph {
        let sm = player.get_state_machine();
        let read_lock = sm.try_read();

        match read_lock {
            Ok(locked_machine) => {
                let optional_machine = &*locked_machine;

                if let Some(machine_engine) = optional_machine {
                    let machine = machine_engine.get_state_machine();

                    let states = machine.states();
                    let mut nodes = Vec::new();
                    let mut edges = Vec::new();

                    for (i, state) in states.iter().enumerate() {
                        let x = (i % 3) as f64 * 0.3 + 0.2;
                        let y = (i / 3) as f64 * 0.3 + 0.2;
                        nodes.push(Node {
                            x,
                            y,
                            label: state.get_name(),
                            active: machine_engine.get_current_state_name() == state.get_name(),
                        });

                        for transition in state.get_transitions() {
                            let target = machine.get_state_by_name(&transition.get_target_state());
                            if let Some(target) = target {
                                let target_index = states
                                    .iter()
                                    .position(|s| s.get_name() == target.get_name())
                                    .unwrap();
                                edges.push(Edge {
                                    from: i,
                                    to: target_index,
                                });
                            }
                        }
                    }

                    return Graph { nodes, edges };
                } else {
                    println!("Error: State machine is None");
                    process::abort();
                }
            }
            Err(err) => {
                println!("Error: {}", err);
                process::abort();
            }
        }
    }
}

struct GraphWidget<'a> {
    graph: &'a Graph,
    block: Option<Block<'a>>,
}

impl<'a> GraphWidget<'a> {
    fn new(graph: &'a Graph) -> Self {
        GraphWidget { graph, block: None }
    }

    fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl<'a> Widget for GraphWidget<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let graph_area = match self.block {
            Some(b) => {
                let inner_area = b.inner(area);
                b.render(area, buf);
                inner_area
            }
            None => area,
        };

        // Draw edges
        for edge in &self.graph.edges {
            let from = &self.graph.nodes[edge.from];
            let to = &self.graph.nodes[edge.to];
            let start_x = (from.x * graph_area.width as f64) as u16 + graph_area.left();
            let start_y = (from.y * graph_area.height as f64) as u16 + graph_area.top();
            let end_x = (to.x * graph_area.width as f64) as u16 + graph_area.left();
            let end_y = (to.y * graph_area.height as f64) as u16 + graph_area.top();

            // Simple line drawing algorithm
            let dx = (end_x as i32 - start_x as i32).abs();
            let dy = (end_y as i32 - start_y as i32).abs();
            let sx = if start_x < end_x { 1 } else { -1 };
            let sy = if start_y < end_y { 1 } else { -1 };
            let mut err = if dx > dy { dx } else { -dy } / 2;
            let mut current_x = start_x as i32;
            let mut current_y = start_y as i32;

            loop {
                if current_x >= graph_area.left() as i32
                    && current_x < graph_area.right() as i32
                    && current_y >= graph_area.top() as i32
                    && current_y < graph_area.bottom() as i32
                {
                    buf.get_mut(current_x as u16, current_y as u16)
                        .set_char('-')
                        .set_style(Style::default().fg(Color::Gray));
                }
                if current_x == end_x as i32 && current_y == end_y as i32 {
                    break;
                }
                let e2 = err;
                if e2 > -dx {
                    err -= dy;
                    current_x += sx;
                }
                if e2 < dy {
                    err += dx;
                    current_y += sy;
                }
            }
        }

        // Draw nodes
        for node in &self.graph.nodes {
            let x = (node.x * graph_area.width as f64) as u16 + graph_area.left();
            let y = (node.y * graph_area.height as f64) as u16 + graph_area.top();
            if x < graph_area.right() && y < graph_area.bottom() {
                // Draw the node as a small circle
                if node.active {
                    buf.get_mut(x, y)
                        .set_char('●')
                        .set_style(Style::default().fg(Color::Green));
                }
                // Draw the label with a box
                if x + 2 < graph_area.right() && y > graph_area.top() && y + 2 < graph_area.bottom()
                {
                    let label_width = node.label.len().min((graph_area.right() - x - 2) as usize);

                    // Draw top border of the box
                    buf.get_mut(x + 1, y - 1)
                        .set_char('┌')
                        .set_style(Style::default().fg(Color::White));
                    for i in 0..label_width {
                        buf.get_mut(x + 2 + i as u16, y - 1)
                            .set_char('─')
                            .set_style(Style::default().fg(Color::White));
                    }
                    buf.get_mut(x + 2 + label_width as u16, y - 1)
                        .set_char('┐')
                        .set_style(Style::default().fg(Color::White));

                    // Draw the label
                    buf.get_mut(x + 1, y)
                        .set_char('│')
                        .set_style(Style::default().fg(Color::White));
                    for (i, ch) in node.label.chars().take(label_width).enumerate() {
                        buf.get_mut(x + 2 + i as u16, y)
                            .set_char(ch)
                            .set_style(Style::default().fg(Color::White));
                    }
                    buf.get_mut(x + 2 + label_width as u16, y)
                        .set_char('│')
                        .set_style(Style::default().fg(Color::White));

                    // Draw bottom border of the box
                    buf.get_mut(x + 1, y + 1)
                        .set_char('└')
                        .set_style(Style::default().fg(Color::White));
                    for i in 0..label_width {
                        buf.get_mut(x + 2 + i as u16, y + 1)
                            .set_char('─')
                            .set_style(Style::default().fg(Color::White));
                    }
                    buf.get_mut(x + 2 + label_width as u16, y + 1)
                        .set_char('┘')
                        .set_style(Style::default().fg(Color::White));
                }
            }
        }
    }
}

struct LogMessage {
    content: String,
    level: LogLevel,
}

enum LogLevel {
    Info,
    Warning,
    Error,
}
struct Logger {
    messages: Vec<LogMessage>,
    receiver: Receiver<LogMessage>,
}

impl Logger {
    fn new(receiver: Receiver<LogMessage>) -> Self {
        Logger {
            messages: Vec::new(),
            receiver,
        }
    }

    fn update(&mut self) {
        while let Ok(message) = self.receiver.try_recv() {
            self.messages.push(message);
            if self.messages.len() > 100 {
                self.messages.remove(0);
            }
        }
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

    /* Dotlottie stuff ---------------------------------------------------------------------------------------- */
    let lottie_player: DotLottiePlayer = DotLottiePlayer::new(Config {
        background_color: 0xffffffff,
        ..Config::default()
    });

    let mut markers = File::open("./src/bin/new-format/pigeon.lottie").expect("no file found");
    let metadatamarkers =
        fs::metadata("./src/bin/new-format/pigeon.lottie").expect("unable to read metadata");
    let mut markers_buffer = vec![0; metadatamarkers.len() as usize];
    markers.read(&mut markers_buffer).expect("buffer overflow");

    lottie_player.load_dotlottie_data(&markers_buffer, WIDTH as u32, HEIGHT as u32);

    let mut timer = Timer::new();

    let message: String =
        fs::read_to_string(format!("./src/bin/tui/{}.json", LOADED_STATE_MACHINE)).unwrap();

    let r = lottie_player.load_state_machine_data(&message);

    let s = lottie_player.start_state_machine();

    lottie_player.render();

    /* Dotlottie stuff ---------------------------------------------------------------------------------------- */

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let (log_sender, log_receiver) = mpsc::channel();
    let logger = Logger::new(log_receiver);

    log_sender
        .send(LogMessage {
            content: format!("Load state machine data returned: {}", r),
            level: LogLevel::Info,
        })
        .unwrap();
    log_sender
        .send(LogMessage {
            content: format!("Start state machine returned: {}", s),
            level: LogLevel::Info,
        })
        .unwrap();

    run_app(
        &mut terminal,
        &mut window,
        &mut buffer,
        logger,
        log_sender,
        &mut timer,
        &lottie_player,
    )?;

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
    mut logger: Logger,
    log_sender: Sender<LogMessage>,
    timer: &mut Timer,
    player: &DotLottiePlayer,
) -> io::Result<()> {
    let sm = player.get_state_machine();
    let read_lock = sm.try_read();
    let mut triggers: Vec<Trigger> = Vec::new();
    let mut trigger_buttons: Vec<MenuItemType> = Vec::new();

    match read_lock {
        Ok(locked_machine) => {
            let optional_machine = &*locked_machine;

            if let Some(machine_engine) = optional_machine {
                let machine = machine_engine.get_state_machine();

                let triggers_opt = machine.triggers();
                if let Some(triggers_opt) = triggers_opt {
                    triggers = triggers_opt.to_vec();

                    for trigger in &triggers {
                        match trigger {
                            Trigger::String { name, value } => {
                                let mut new_name = name.clone();
                                new_name = format!("[String] {}", new_name);

                                trigger_buttons.push(MenuItemType::StringInput {
                                    name: new_name.to_string(),
                                    value: value.to_string(),
                                });
                            }
                            Trigger::Boolean { name, value } => {
                                let mut new_name = name.clone();
                                new_name = format!("[Bool] {}", new_name);

                                trigger_buttons.push(MenuItemType::BooleanToggle {
                                    name: new_name.to_string(),
                                    value: *value,
                                });
                            }
                            Trigger::Numeric { name, value } => {
                                let mut new_name = name.clone();
                                new_name = format!("[Numeric] {}", new_name);

                                trigger_buttons.push(MenuItemType::NumberInput {
                                    name: new_name.to_string(),
                                    value: *value,
                                    buffer: value.to_string(),
                                });
                            }
                            Trigger::Event { name } => {
                                let mut new_name = name.clone();
                                new_name = format!("[Event] {}", new_name);

                                trigger_buttons.push(MenuItemType::Button {
                                    name: new_name.to_string(),
                                    color: 0x000000,
                                });
                            }
                        }
                    }
                }
            }
        }
        Err(err) => {
            println!("Error: {}", err);
            process::abort();
        }
    }

    let mut menus = vec![
        Menu::new(
            "🚧 [Unavailable]".to_string(),
            vec![
                // MenuItemType::Button {
                //     name: "Red".to_string(),
                //     color: 0xFF0000,
                // },
                // MenuItemType::Button {
                //     name: "Green".to_string(),
                //     color: 0x00FF00,
                // },
                // MenuItemType::Button {
                //     name: "Blue".to_string(),
                //     color: 0x0000FF,
                // },
            ],
        ),
        Menu::new("Triggers".to_string(), trigger_buttons),
        Menu::new(
            "🚧 [Unavailable] Actions".to_string(),
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

    let mut graph = Graph::new(&player);

    let mut current_menu = 0;
    let mut selected_color = 0xFF0000;
    let mut last_update = Instant::now();
    let mut input_mode = false;

    log_sender
        .send(LogMessage {
            content: "Application started".to_string(),
            level: LogLevel::Info,
        })
        .unwrap();

    loop {
        logger.update();
        graph = graph.update(player);

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Percentage(20),
                        Constraint::Percentage(20),
                        Constraint::Percentage(20),
                        Constraint::Percentage(20),
                        Constraint::Percentage(20),
                    ]
                    .as_ref(),
                )
                .split(f.size());

            /* This section performs rendering to the terminal  */
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
                                        if *color == selected_color { "> " } else { "" },
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
                            MenuItemType::NumberInput {
                                name,
                                value,
                                buffer,
                            } => {
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
                                (format!("{}: {}", name, buffer), style) // Display the buffer instead of the value
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
                                    format!("{}: {}", name, if *value { "True" } else { "False" }),
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

            // Render the graph
            let graph_widget = GraphWidget::new(&graph).block(
                Block::default()
                    .title(LOADED_STATE_MACHINE)
                    .borders(Borders::ALL),
            );
            f.render_widget(graph_widget, chunks[3]);

            // Render the log area
            let log_widget = Paragraph::new(Text::from(
                logger
                    .messages
                    .iter()
                    .map(|msg| {
                        let (content, style) = match msg.level {
                            LogLevel::Info => (
                                format!("[INFO] {}", msg.content),
                                Style::default().fg(Color::Green),
                            ),
                            LogLevel::Warning => (
                                format!("[WARN] {}", msg.content),
                                Style::default().fg(Color::Yellow),
                            ),
                            LogLevel::Error => (
                                format!("[ERROR] {}", msg.content),
                                Style::default().fg(Color::Red),
                            ),
                        };
                        Spans::from(vec![Span::styled(content, style)])
                    })
                    .collect::<Vec<Spans>>(),
            ))
            .block(Block::default().title("Logs").borders(Borders::ALL))
            .wrap(ratatui::widgets::Wrap { trim: true });
            f.render_widget(log_widget, chunks[4]);
        })?;

        if last_update.elapsed() >= Duration::from_millis(16) {
            timer.tick(player);

            let (buffer_ptr, buffer_len) = (player.buffer_ptr(), player.buffer_len());

            let buffer = unsafe {
                std::slice::from_raw_parts(buffer_ptr as *const u32, buffer_len as usize)
            };

            window.update_with_buffer(buffer, WIDTH, HEIGHT).unwrap();
            last_update = Instant::now();
        }

        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if input_mode {
                    handle_input_mode(&mut menus[current_menu], key);
                    if key.code == KeyCode::Esc {
                        send_input_to_state_machine(&mut menus[current_menu], key, player);

                        log_sender
                            .send(LogMessage {
                                content: "Exited input mode".to_string(),
                                level: LogLevel::Info,
                            })
                            .unwrap();
                        input_mode = false;
                    }
                } else {
                    match key.code {
                        KeyCode::Char('q') => {
                            log_sender
                                .send(LogMessage {
                                    content: "User initiated quit".to_string(),
                                    level: LogLevel::Info,
                                })
                                .unwrap();
                            return Ok(());
                        }
                        KeyCode::Down => {
                            move_selection(&mut menus, &mut current_menu, 1);
                        }
                        KeyCode::Up => {
                            move_selection(&mut menus, &mut current_menu, -1);
                        }
                        KeyCode::Enter => {
                            let menu = &mut menus[current_menu];
                            let i = menu.state.selected().unwrap_or(0);
                            match &mut menu.items[i] {
                                MenuItemType::Button { name, color } => match name.as_str() {
                                    "Quit" => {
                                        log_sender
                                            .send(LogMessage {
                                                content: "User selected Quit".to_string(),
                                                level: LogLevel::Info,
                                            })
                                            .unwrap();
                                        return Ok(());
                                    }
                                    "Clear" => {
                                        player.state_machine_fire_event("Step");
                                    }
                                    "Random" => {
                                        log_sender
                                            .send(LogMessage {
                                                content: "Set random color".to_string(),
                                                level: LogLevel::Info,
                                            })
                                            .unwrap();
                                        selected_color = rand::random::<u32>() | 0xFF000000;
                                    }
                                    _ => {
                                        let new_name = name.replace("[Event] ", "");

                                        log_sender
                                            .send(LogMessage {
                                                content: format!("Firing event: {}", new_name),
                                                level: LogLevel::Info,
                                            })
                                            .unwrap();
                                        player.state_machine_fire_event(&new_name);
                                    }
                                },
                                MenuItemType::StringInput { name, value } => {
                                    log_sender
                                        .send(LogMessage {
                                            content: "Entered input mode".to_string(),
                                            level: LogLevel::Info,
                                        })
                                        .unwrap();
                                    input_mode = true;
                                }
                                MenuItemType::NumberInput {
                                    name,
                                    value,
                                    buffer,
                                } => {
                                    log_sender
                                        .send(LogMessage {
                                            content: "Entered input mode".to_string(),
                                            level: LogLevel::Info,
                                        })
                                        .unwrap();
                                    input_mode = true;
                                }
                                MenuItemType::BooleanToggle { value, .. } => {
                                    *value = !*value;
                                    log_sender
                                        .send(LogMessage {
                                            content: format!("Toggled boolean to {}", *value),
                                            level: LogLevel::Info,
                                        })
                                        .unwrap();
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
        MenuItemType::NumberInput { value, buffer, .. } => match key.code {
            KeyCode::Char(c) if c.is_digit(10) || c == '.' => {
                buffer.push(c);
                if let Ok(new_value) = buffer.parse::<f32>() {
                    *value = new_value;
                }
            }
            KeyCode::Char('-') if buffer.is_empty() => {
                buffer.push('-');
            }
            KeyCode::Backspace => {
                buffer.pop();
                *value = buffer.parse::<f32>().unwrap_or(0.0);
            }
            _ => {}
        },
        _ => {}
    }
}

fn send_input_to_state_machine(menu: &mut Menu, key: event::KeyEvent, player: &DotLottiePlayer) {
    let i = menu.state.selected().unwrap_or(0);
    match &mut menu.items[i] {
        MenuItemType::StringInput { value, name } => {
            let new_name = name.replace("[String] ", "");
            player.state_machine_set_string_trigger(&new_name, value);
        }
        MenuItemType::NumberInput {
            value,
            name,
            buffer,
        } => {
            let new_name = name.replace("[Numeric] ", "");
            player.state_machine_set_numeric_trigger(&new_name, *value);
        }
        MenuItemType::BooleanToggle { value, name } => {
            let new_name = name.replace("[Bool] ", "");
            player.state_machine_set_boolean_trigger(&new_name, *value);
        }
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

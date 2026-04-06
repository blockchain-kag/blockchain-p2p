use std::{
    io::{Stdout, Write, stdout},
    ops::Add,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, Sender, channel},
    },
    thread::sleep,
    time::Duration,
};

use blockchain_p2p::{
    consensus_engine::types::consensus_engine::ConsensusEngine,
    mempool::types::mempool::Mempool,
    node::{
        adapters::{
            consensus_engine::{miner::CpuMiner, multiple_validator::MultipleValidator},
            hasher::sha256_hasher::Sha256Hasher,
            storage::in_memory_storage::InMemoryStorage,
        },
        types::{node::Node, node_event::NodeEvent, node_event_handler::NodeEventHandler},
    },
};
use crossterm::{
    cursor::{MoveDown, MoveLeft, MoveTo},
    event::{Event, KeyCode, poll, read},
    execute, queue,
    style::Print,
    terminal::{self, Clear, ClearType, WindowSize},
};

fn main() {
    let (in_tx, in_rx) = channel::<String>();
    let (event_tx, event_rx) = channel::<NodeEvent>();
    let (out_tx, out_rx) = channel::<String>();
    let _input_handle = NodeEventHandler::new(in_rx, event_tx).run();
    let mut stdout = stdout();
    let mut w_size: WindowSize = terminal::window_size().unwrap();
    let mut prompt = String::new();
    let mut outputs: Vec<String> = vec![];
    terminal::enable_raw_mode().unwrap();
    let shutdown = Arc::new(AtomicBool::from(false));
    let mempool = Mempool::new();
    let storage = Box::new(InMemoryStorage::new());
    let difficulty = 3;
    let hasher = Arc::new(Sha256Hasher);
    let miner = Box::new(CpuMiner::new(hasher.clone()));
    let validator = Box::new(MultipleValidator::new());
    let consensus_engine = ConsensusEngine::new(miner, validator, difficulty);
    let _node_handle = Node::new(
        event_rx,
        shutdown.clone(),
        out_tx,
        mempool,
        storage,
        consensus_engine,
        hasher.clone(),
    )
    .run();
    while !shutdown.clone().load(Ordering::Relaxed) {
        handle_user_interaction(&in_tx, &mut w_size, &mut prompt, shutdown.clone());
        render_screen(&out_rx, &mut stdout, &w_size, &prompt, &mut outputs);
    }
    render_screen(&out_rx, &mut stdout, &w_size, &prompt, &mut outputs);

    sleep(Duration::from_secs(1));
    terminal::disable_raw_mode().unwrap();
    execute!(stdout, terminal::Clear(ClearType::All), MoveTo(0, 0)).unwrap();
}

fn handle_user_interaction(
    in_tx: &Sender<String>,
    w_size: &mut WindowSize,
    prompt: &mut String,
    quit: Arc<AtomicBool>,
) {
    while poll(Duration::ZERO).unwrap() {
        match read().unwrap() {
            Event::Key(key_event) => match key_event.code {
                KeyCode::Backspace => {
                    prompt.pop();
                }
                KeyCode::Enter => {
                    in_tx.send(prompt.clone()).unwrap();
                    *prompt = String::new();
                }
                KeyCode::Left => todo!(),
                KeyCode::Right => todo!(),
                KeyCode::Up => todo!(),
                KeyCode::Down => todo!(),
                KeyCode::Char(c) => {
                    prompt.push(c);
                }
                KeyCode::Esc => {
                    quit.store(true, Ordering::Release);
                }
                _ => {}
            },
            Event::Paste(data) => {
                in_tx.send(prompt.clone()).unwrap();
                *prompt = prompt.clone().add(&data);
            }
            Event::Resize(w, h) => {
                *w_size = WindowSize {
                    rows: h,
                    columns: w,
                    width: w_size.width,
                    height: w_size.height,
                }
            }
            _ => {}
        }
    }
}

fn render_screen(
    out_rx: &Receiver<String>,
    stdout: &mut Stdout,
    w_size: &WindowSize,
    prompt: &str,
    outputs: &mut Vec<String>,
) {
    queue!(stdout, Clear(ClearType::All)).unwrap();

    if let Ok(output) = out_rx.try_recv() {
        outputs.push(output);
    }

    let bd_left = 0;
    let bd_top = 0;
    let bd_right = w_size.columns - 1;
    let bd_bottom = w_size.rows - 1;

    let content_padding = 1;
    let main_section_inner_top = bd_top + 3;
    let main_section_inner_bottom = bd_bottom - 2;

    let screen_division = w_size.columns / 2;

    outer_box_rendering(
        stdout,
        bd_left,
        bd_top,
        bd_right,
        bd_bottom,
        screen_division,
    );
    queue_section_title(1, 0, "System messages section");
    queue_section_title(screen_division + 1, 0, "Minning section");

    entries_renderer(
        &*outputs,
        main_section_inner_top,
        screen_division,
        main_section_inner_bottom,
        bd_left,
        content_padding,
    );
    let sections = vec![
        vec![
            String::from("Status: "),
            String::from("Hash rate: "),
            String::from("Difficulty: "),
        ],
        vec![
            String::from("Block: "),
            String::from("Nonce: "),
            String::from("Attempts: "),
        ],
        vec![
            String::from("Last block: "),
            String::from("Hash: "),
            String::from("Time: "),
        ],
    ];

    static_section_rendering(
        screen_division,
        main_section_inner_top,
        content_padding,
        sections,
    );

    let prompt_start = if prompt.len() >= bd_right as usize {
        prompt.len() - (bd_right as usize - 1)
    } else {
        0
    };

    queue!(
        stdout,
        MoveTo(bd_left + 1, bd_bottom - 1),
        Print(prompt.get(prompt_start..prompt.len()).unwrap())
    )
    .unwrap();

    stdout.flush().unwrap();
    sleep(Duration::from_millis(33));
}

fn outer_box_rendering(
    stdout: &mut Stdout,
    bd_left: u16,
    bd_top: u16,
    bd_right: u16,
    bd_bottom: u16,
    screen_division: u16,
) {
    let prompt_start = bd_bottom - 2;
    for y in 1..=bd_bottom {
        queue!(stdout, MoveTo(bd_left, y), Print("║")).unwrap();
        if y < prompt_start {
            queue!(stdout, MoveTo(screen_division, y), Print("║")).unwrap();
        }
        queue!(stdout, MoveTo(bd_right, y), Print("║")).unwrap();
    }
    for x in 0..=bd_right {
        let border_top = bd_top + 1;
        let border_med = prompt_start;
        let border_bot = bd_bottom;
        if x == bd_left {
            queue!(stdout, MoveTo(x, border_top), Print("╔")).unwrap();
            queue!(stdout, MoveTo(x, border_med), Print("╠")).unwrap();
            queue!(stdout, MoveTo(x, border_bot), Print("╚")).unwrap();
        } else if x == screen_division {
            queue!(stdout, MoveTo(x, border_top), Print("╦")).unwrap();
            queue!(stdout, MoveTo(x, border_med), Print("╩")).unwrap();
            queue!(stdout, MoveTo(x, border_bot), Print("═")).unwrap();
        } else if x == bd_right {
            queue!(stdout, MoveTo(x, border_top), Print("╗")).unwrap();
            queue!(stdout, MoveTo(x, border_med), Print("╣")).unwrap();
            queue!(stdout, MoveTo(x, border_bot), Print("╝")).unwrap();
        } else {
            queue!(stdout, MoveTo(x, border_top), Print("═")).unwrap();
            queue!(stdout, MoveTo(x, border_med), Print("═")).unwrap();
            queue!(stdout, MoveTo(x, border_bot), Print("═")).unwrap();
        }
    }
}

fn static_section_rendering(
    left_boundary: u16,
    top_boundary: u16,
    padding: u16,
    sections: Vec<Vec<String>>,
) {
    let mut stdout = stdout();
    queue!(
        stdout,
        MoveTo(left_boundary + 1 + padding, top_boundary + padding)
    )
    .unwrap();

    for section in sections {
        for field in section {
            queue!(
                stdout,
                Print(&field),
                MoveDown(2),
                MoveLeft(field.len() as u16)
            )
            .unwrap()
        }
        queue!(stdout, MoveDown(1)).unwrap();
    }
}

fn entries_renderer(
    outputs: &[String],
    top_boundary: u16,
    right_boundary: u16,
    bottom_boundary: u16,
    left_boundary: u16,
    padding: u16,
) {
    let height = bottom_boundary - top_boundary - 1 - 2 * padding;
    let width = right_boundary - left_boundary - 1 - 2 * padding;
    let mut stdout = stdout();
    let skip = outputs.len().saturating_sub(height as usize);
    for (dy, output) in outputs.iter().skip(skip).enumerate() {
        queue!(
            stdout,
            MoveTo(
                left_boundary + 1 + padding,
                top_boundary + padding + dy as u16
            ),
            Print(output.get(0..width as usize).unwrap_or(output))
        )
        .unwrap();
    }
}

fn queue_section_title(x_start: u16, y_start: u16, title: &str) {
    let mut stdout = stdout();
    let title_container_side = "─".repeat(title.len());
    queue!(
        stdout,
        MoveTo(x_start, y_start),
        Print(format!("{}{title_container_side}{}", "┌", "┐")),
        MoveTo(x_start, y_start + 1),
        Print(format!("{}{title}{}", "╡", "╞")),
        MoveTo(x_start, y_start + 2),
        Print(format!("{}{title_container_side}{}", "└", "┘")),
    )
    .unwrap();
}

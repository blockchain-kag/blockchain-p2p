use std::{
    io::{Write, stdout},
    ops::Add,
    sync::mpsc::channel,
    thread::sleep,
    time::Duration,
};

use crossterm::{
    cursor::{MoveDown, MoveLeft, MoveTo},
    event::{poll, read},
    execute, queue,
    style::Print,
    terminal::{self, Clear, ClearType, WindowSize},
};

fn main() {
    let (in_tx, out_rx) = channel::<String>();
    let (_, _) = channel::<String>();
    let mut stdout = stdout();
    let mut w_size: WindowSize = terminal::window_size().unwrap();
    let mut prompt = String::new();
    let mut outputs: Vec<String> = vec![];
    terminal::enable_raw_mode().unwrap();
    let mut quit = false;
    while !quit {
        while poll(Duration::ZERO).unwrap() {
            match read().unwrap() {
                crossterm::event::Event::Key(key_event) => match key_event.code {
                    crossterm::event::KeyCode::Backspace => {
                        prompt.pop();
                    }
                    crossterm::event::KeyCode::Enter => {
                        in_tx.send(prompt.clone()).unwrap();
                        prompt = String::new();
                    }
                    crossterm::event::KeyCode::Left => todo!(),
                    crossterm::event::KeyCode::Right => todo!(),
                    crossterm::event::KeyCode::Up => todo!(),
                    crossterm::event::KeyCode::Down => todo!(),
                    crossterm::event::KeyCode::Char(c) => {
                        prompt.push(c);
                    }
                    crossterm::event::KeyCode::Esc => {
                        quit = true;
                    }
                    _ => {}
                },
                crossterm::event::Event::Paste(data) => {
                    in_tx.send(prompt.clone()).unwrap();
                    prompt = prompt.add(&data);
                }
                crossterm::event::Event::Resize(w, h) => {
                    w_size = WindowSize {
                        rows: h,
                        columns: w,
                        width: w_size.width,
                        height: w_size.height,
                    }
                }
                _ => {}
            }
        }

        queue!(stdout, Clear(ClearType::All)).unwrap();

        if let Ok(output) = out_rx.try_recv() {
            outputs.push(output);
        }

        let prompt_start = w_size.rows - 3;
        let screen_division = w_size.columns / 2;

        let inner_top = 3;
        let inner_right = w_size.columns - 2;
        let inner_bottom = w_size.rows - 2;
        let inner_left = 0;

        let content_padding = 1;
        let main_section_iner = inner_bottom - 2;

        for y in 1..w_size.rows - 1 {
            queue!(stdout, MoveTo(0, y), Print("║")).unwrap();
            if y != w_size.rows - 2 {
                queue!(stdout, MoveTo(screen_division, y), Print("║")).unwrap();
            }
            queue!(stdout, MoveTo(w_size.columns - 1, y), Print("║")).unwrap();
        }
        for x in 0..w_size.columns {
            if x == 0 {
                queue!(stdout, MoveTo(x, 1), Print("╔")).unwrap();
                queue!(stdout, MoveTo(x, prompt_start), Print("╠")).unwrap();
                queue!(stdout, MoveTo(x, w_size.rows - 1), Print("╚")).unwrap();
            } else if x == screen_division {
                queue!(stdout, MoveTo(x, 1), Print("╦")).unwrap();
                queue!(stdout, MoveTo(x, prompt_start), Print("╩")).unwrap();
                queue!(stdout, MoveTo(x, w_size.rows - 1), Print("═")).unwrap();
            } else if x == w_size.columns - 1 {
                queue!(stdout, MoveTo(x, 1), Print("╗")).unwrap();
                queue!(stdout, MoveTo(x, prompt_start), Print("╣")).unwrap();
                queue!(stdout, MoveTo(x, w_size.rows - 1), Print("╝")).unwrap();
            } else {
                queue!(stdout, MoveTo(x, 1), Print("═")).unwrap();
                queue!(stdout, MoveTo(x, prompt_start), Print("═")).unwrap();
                queue!(stdout, MoveTo(x, w_size.rows - 1), Print("═")).unwrap();
            }
        }
        queue_section_title(1, 0, "System messages section");
        queue_section_title(screen_division + 1, 0, "Minning section");

        entries_renderer(
            &outputs,
            inner_top,
            screen_division,
            main_section_iner,
            inner_left,
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

        static_section_rendering(screen_division, inner_top, content_padding, sections);

        let prompt_start = if prompt.len() > w_size.columns as usize {
            prompt.len() - (w_size.columns as usize - 2)
        } else {
            0
        };

        queue!(
            stdout,
            MoveTo(1, w_size.rows - 2),
            Print(prompt.get(prompt_start..prompt.len()).unwrap())
        )
        .unwrap();

        stdout.flush().unwrap();
        sleep(Duration::from_millis(33));
    }
    terminal::disable_raw_mode().unwrap();
    execute!(stdout, terminal::Clear(ClearType::All), MoveTo(0, 0)).unwrap();
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
    let height = bottom_boundary - top_boundary + 1 - 2 * padding;
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

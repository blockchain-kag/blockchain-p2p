use std::{
    io::{Write, stdout},
    ops::Add,
    thread::sleep,
    time::Duration,
};

use crossterm::{
    cursor::MoveTo,
    event::{poll, read},
    queue,
    style::Print,
    terminal::{self, Clear, ClearType, WindowSize},
};

fn main() {
    let mut stdout = stdout();
    let mut w_size: WindowSize = terminal::window_size().unwrap();
    let mut prompt = String::new();
    terminal::enable_raw_mode().unwrap();
    let mut quit = false;
    while !quit {
        while poll(Duration::ZERO).unwrap() {
            match read().unwrap() {
                crossterm::event::Event::FocusGained => todo!(),
                crossterm::event::Event::FocusLost => todo!(),
                crossterm::event::Event::Key(key_event) => match key_event.code {
                    crossterm::event::KeyCode::Backspace => {
                        prompt.pop();
                    }
                    crossterm::event::KeyCode::Enter => todo!(),
                    crossterm::event::KeyCode::Left => todo!(),
                    crossterm::event::KeyCode::Right => todo!(),
                    crossterm::event::KeyCode::Up => todo!(),
                    crossterm::event::KeyCode::Down => todo!(),
                    crossterm::event::KeyCode::Home => todo!(),
                    crossterm::event::KeyCode::End => todo!(),
                    crossterm::event::KeyCode::PageUp => todo!(),
                    crossterm::event::KeyCode::PageDown => todo!(),
                    crossterm::event::KeyCode::Tab => todo!(),
                    crossterm::event::KeyCode::BackTab => todo!(),
                    crossterm::event::KeyCode::Delete => todo!(),
                    crossterm::event::KeyCode::Insert => todo!(),
                    crossterm::event::KeyCode::F(_) => todo!(),
                    crossterm::event::KeyCode::Char(c) => {
                        prompt.push(c);
                    }
                    crossterm::event::KeyCode::Null => todo!(),
                    crossterm::event::KeyCode::Esc => {
                        quit = true;
                    }
                    crossterm::event::KeyCode::CapsLock => todo!(),
                    crossterm::event::KeyCode::ScrollLock => todo!(),
                    crossterm::event::KeyCode::NumLock => todo!(),
                    crossterm::event::KeyCode::PrintScreen => todo!(),
                    crossterm::event::KeyCode::Pause => todo!(),
                    crossterm::event::KeyCode::Menu => todo!(),
                    crossterm::event::KeyCode::KeypadBegin => todo!(),
                    crossterm::event::KeyCode::Media(_) => todo!(),
                    crossterm::event::KeyCode::Modifier(_) => todo!(),
                },
                crossterm::event::Event::Mouse(_) => todo!(),
                crossterm::event::Event::Paste(data) => {
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
            }
        }
        let prompt_start = w_size.rows - 3;
        let screen_division = w_size.columns / 2;
        queue!(stdout, Clear(ClearType::All)).unwrap();
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

        queue!(stdout, MoveTo(1, w_size.rows - 2), Print(&prompt)).unwrap();

        stdout.flush().unwrap();
        sleep(Duration::from_millis(33));
    }
    terminal::disable_raw_mode().unwrap();
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

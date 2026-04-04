use std::{
    io::{Write, stdout},
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
                crossterm::event::Event::Paste(_) => todo!(),
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

        queue!(stdout, Clear(ClearType::All)).unwrap();
        for y in 0..(w_size.rows - 2) {
            queue!(stdout, MoveTo(w_size.columns / 2, y), Print("║")).unwrap();
        }
        for x in 0..w_size.columns {
            if x == w_size.columns / 2 {
                queue!(
                    stdout,
                    MoveTo(w_size.columns / 2, w_size.rows - 2),
                    Print("╩")
                )
                .unwrap();
            } else {
                queue!(stdout, MoveTo(x, w_size.rows - 2), Print("═")).unwrap();
            }
        }

        queue!(stdout, MoveTo(0, w_size.rows), Print(&prompt)).unwrap();

        stdout.flush().unwrap();
        sleep(Duration::from_millis(33));
    }
    terminal::disable_raw_mode().unwrap();
}

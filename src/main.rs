use std::{
    io::{Write, stdout},
    thread::sleep,
    time::Duration,
};

use crossterm::{
    QueueableCommand,
    cursor::{self, MoveDown, MoveRight, MoveTo, MoveToColumn, MoveToRow},
    execute, queue,
    style::{self, Print, Stylize, style},
    terminal::{self, Clear, ClearType, WindowSize},
};

fn main() {
    let mut stdout = stdout();
    let w_size: WindowSize = terminal::window_size().unwrap();
    loop {
        queue!(stdout, Clear(ClearType::All)).unwrap();
        for y in 0..(w_size.rows - 2) {
            queue!(stdout, MoveTo(w_size.columns / 2, y), Print("║")).unwrap();
        }
        for x in 0..w_size.columns {
            queue!(stdout, MoveTo(x, w_size.rows - 2), Print("═")).unwrap();
        }

        queue!(
            stdout,
            MoveTo(w_size.columns / 2, w_size.rows - 2),
            Print("╩")
        )
        .unwrap();

        stdout.flush().unwrap();
        sleep(Duration::from_secs(1));
    }
}

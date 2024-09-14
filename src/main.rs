use std::io;
use std::io::Write;
use std::time::Duration;

use crossterm::cursor;
use crossterm::event::poll;
use crossterm::execute;
use crossterm::queue;
use crossterm::style::Print;
use crossterm::terminal::{
    size,
    Clear,
    ClearType,
    enable_raw_mode,
    disable_raw_mode,
    EnterAlternateScreen,
    LeaveAlternateScreen,
    DisableLineWrap,
    EnableLineWrap,
};
use crossterm::event::{
    read,
    Event,
    KeyEvent,
    KeyCode,
};

use rand::prelude::*;

static FPS: u64 = 60;
static FRAME_SKIPS: u64 = 5;

struct GameState {
    should_quit: bool,
    snake: Snake,
    width: u16,
    height: u16,
    keydown: Option<char>,
    food: (u16, u16),
}

#[derive(PartialEq)]
enum Direction { Up, Down, Left, Right }

struct Snake {
    segments: Vec<(u16, u16)>,
    direction: Direction,
}

fn determine_food_location(state: &GameState) -> (u16, u16) {
    let mut rng = rand::thread_rng();
    (rng.gen_range(0..state.width-1), rng.gen_range(0..state.height-1))
}

fn draw(state: &GameState) {
    queue!(
        io::stdout(),
        Clear(ClearType::All),
        cursor::MoveTo(state.food.0, state.food.1),
        Print('x'),
    ).ok();

    for segment in state.snake.segments.iter() {
        queue!(
            io::stdout(),
            cursor::MoveTo(segment.0, segment.1),
            Print('o')
        ).ok();
    }

    io::stdout().flush().ok();
}

fn update(state: &mut GameState) {
    // Handle input
    if let Some(c) = state.keydown {
        match c {
            'w' => {
                if state.snake.direction != Direction::Down {
                    state.snake.direction = Direction::Up
                }
            },
            'a' => {
                if state.snake.direction != Direction::Right {
                    state.snake.direction = Direction::Left
                }
            },
            's' => {
                if state.snake.direction != Direction::Up {
                    state.snake.direction = Direction::Down
                }
            },
            'd' => {
                if state.snake.direction != Direction::Left {
                    state.snake.direction = Direction::Right
                }
            },
            _ => {}
        };
    }

    // Using unwrap here, since the game would be broken if there were no head.
    let mut new_head = state.snake.segments.last().unwrap().clone();

    if new_head.0 == state.food.0 && new_head.1 == state.food.1 {
        state.food = determine_food_location(state);
    } else {
        state.snake.segments.remove(0);
    }

    match state.snake.direction {
        Direction::Up => new_head.1 -= 1,
        Direction::Left => new_head.0 -= 1,
        Direction::Down => new_head.1 += 1,
        Direction::Right => new_head.0 += 1,
    }

    state.snake.segments.push(new_head);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (width, height) = size()?;
    let mut snake = Snake {
        segments: vec![(0, 0), (1, 0), (2, 0)],
        direction: Direction::Right,
    };
    let mut frame = 0;
    let mut state = GameState {
        should_quit: false,
        snake,
        width,
        height,
        keydown: None,
        food: (0, 0),
    };
    state.food = determine_food_location(&state);

    enable_raw_mode()?;
    execute!(
        io::stdout(),
        EnterAlternateScreen,
        DisableLineWrap,
        cursor::Hide
    )?;

    while !state.should_quit {
        if poll(Duration::from_millis(1000/FPS))? {
            if let Event::Key(event) = read()? {
                match event.code {
                    KeyCode::Char(c) => {
                        match c {
                            'q' => state.should_quit = true,
                            _ => state.keydown = Some(c),
                        }
                    },
                    _ => {}
                }
            }
        }

        if frame == FRAME_SKIPS {
            draw(&state);
            update(&mut state);
            frame = 0;
        } else {
            frame += 1;
        }
    }

    execute!(
        io::stdout(),
        LeaveAlternateScreen,
        EnableLineWrap,
        cursor::Show,
    )?;
    disable_raw_mode()?;

    Ok(())
}

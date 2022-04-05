extern crate rand;
extern crate rustbox;

use std::time::Duration;
use rand::distributions::{Distribution, Uniform};
use rustbox::{Color, RustBox, InitOptions};

const BOARD_SIZE: usize = 5;
const BOARD_RENDER_LOCATION: (f32, f32, f32, f32) = (18.0, 1.0, 42.0, 21.0);

struct RenderCalculation {
    scaled_table: (f32, f32, f32, f32),
    block_size: (f32, f32)
}

struct Game {
    has_won: bool,
    mouse_released: bool,
    moves: i32,
    board: [[bool; BOARD_SIZE]; BOARD_SIZE],
    render: RenderCalculation
}

fn main() {
    let rustbox = match RustBox::init(InitOptions {
        input_mode: rustbox::InputMode::EscMouse,
        ..Default::default()
    }) {
        Result::Ok(v) => v,
        Result::Err(e) => panic!("Something happened whilst trying to initialize terminal: {}", e),
    };
    
    let mut game: Game = Game { 
        has_won: false,
        mouse_released: true,
        moves: 0,
        board: [[false; BOARD_SIZE]; BOARD_SIZE],
        render: get_render_calculations(rustbox.width(), rustbox.height())
    };

    generate_board(&mut game.board);

    loop {
        rustbox.clear();
        
        // draw routines
        draw_background(&rustbox);
        draw_hud(&game, &rustbox);
        draw_table(&game, &rustbox);
        
        rustbox.present();

        match rustbox.peek_event(Duration::from_millis(1), false) {
            Ok(rustbox::Event::KeyEvent(key)) => {
                match key {
                    rustbox::Key::Char('q') => break,
                    rustbox::Key::Char('r') => { reset(&mut game) }
                    _ => {}
                }
            }
            Ok(rustbox::Event::MouseEvent(mouse, x, y)) => {
                match mouse {
                    rustbox::Mouse::Left => {
                        mouse_left_click(&mut game, x, y);
                    },
                    rustbox::Mouse::Release => {
                        game.mouse_released = true;
                    }
                    _ => {}
                }
            }
            Ok(rustbox::Event::ResizeEvent(w, h)) => {
                game.render = get_render_calculations(w as usize, h as usize)
            }
            Err(e) => panic!("{}", e),
            _ => {}
        }
    }
}

// logic & input handling
fn clear_board(board: &mut [[bool; BOARD_SIZE]; BOARD_SIZE]) {
    for i in 0..BOARD_SIZE {
        for j in 0..BOARD_SIZE {
            board[i][j] = false;
        }
    }
}

fn generate_board(board: &mut [[bool; BOARD_SIZE]; BOARD_SIZE]) {
    clear_board(board);
    let size = BOARD_SIZE as f32;
    let board_press_range = Uniform::from(size*size*0.2..size*size*0.8);
    let board_size_range = Uniform::from(0..BOARD_SIZE*BOARD_SIZE);
    let mut rng = rand::thread_rng();

    let amount_to_press: i32 = board_press_range.sample(&mut rng).round() as i32;
    for _i in 0..amount_to_press {
        let linear_position = board_size_range.sample(&mut rng) as usize;
        press_button(board, linear_position/BOARD_SIZE, linear_position % BOARD_SIZE);
    }
}

fn reset(game: &mut Game) {
    game.has_won = false;
    game.moves = 0;
    generate_board(&mut game.board);
}

fn mouse_left_click(game: &mut Game, x: i32, y: i32) {
    let tbl = get_block_location(game, x as usize, y as usize);
    // check if it's in the table in the first place
    if x as f32 > game.render.scaled_table.0.ceil() && x as f32 <= game.render.scaled_table.0.ceil() + game.render.scaled_table.2.ceil() {
        if y as f32 > game.render.scaled_table.1.ceil() && y as f32 <= game.render.scaled_table.1.ceil() + game.render.scaled_table.3.ceil() {
            if tbl.0 < BOARD_SIZE && tbl.1 < BOARD_SIZE && game.mouse_released && !game.has_won {
                press_button(&mut game.board, tbl.0, tbl.1);
                game.moves += 1;
                game.has_won = game.board.iter().all(|&row| {row.iter().all(|&block| !block)});
            }
        }
    }
    game.mouse_released = false;
}

fn press_button(board: &mut [[bool; BOARD_SIZE]; BOARD_SIZE], x: usize, y: usize) {
    board[x][y] = !board[x][y]; // itself
    if y > 0 { // above
        board[x][y-1] = !board[x][y-1];
    }
    if y < BOARD_SIZE-1 { // below
        board[x][y+1] = !board[x][y+1];
    }
    if x > 0 { // left
        board[x-1][y] = !board[x-1][y];
    }
    if x < BOARD_SIZE-1 { // right
        board[x+1][y] = !board[x+1][y];
    }
}

// rendering
fn get_scale(width: usize, height: usize) -> (f32, f32, f32) {
    let h_scale = width as f32 / 80.0;
    let v_scale = height as f32 / 24.0;
    
    return (h_scale, v_scale, (h_scale*v_scale).sqrt());
}

fn get_render_calculations(width: usize, height: usize) -> RenderCalculation {
    let scale = get_scale(width, height);
    let scaled_table = (BOARD_RENDER_LOCATION.0 * scale.0, BOARD_RENDER_LOCATION.1 * scale.1, BOARD_RENDER_LOCATION.2 * scale.0, BOARD_RENDER_LOCATION.3 * scale.1);
    let block_size = (scaled_table.2 / BOARD_SIZE as f32, (scaled_table.3 - 2.5) / BOARD_SIZE as f32);

    return RenderCalculation { scaled_table: scaled_table, block_size: block_size };
}

fn get_block_render_location(game: &Game, x: usize, y: usize) -> (usize, usize, usize, usize) {
    let x = (x as f32 * game.render.block_size.0.ceil()) + game.render.scaled_table.0.ceil();
    let y = (y as f32 * game.render.block_size.1.ceil()) + game.render.scaled_table.1.ceil();
    let w = game.render.block_size.0.ceil() - 1.0;
    let h = game.render.block_size.1.ceil() - 1.0;

    return (x as usize, y as usize, w as usize, h as usize);
}

fn get_block_location(game: &Game, x: usize, y: usize) -> (usize, usize) {
    let x = (x as f32 - game.render.scaled_table.0.ceil()) / game.render.block_size.0.ceil();
    let y = (y as f32 - game.render.scaled_table.1.ceil()) / game.render.block_size.1.ceil();

    return (x as usize, y as usize);
}

// primitives
fn fill_rect(x: usize, y: usize, w: usize, h: usize, rustbox: &RustBox, color: Color) {
    for i in x..(x + w) {
        for j in y..(y + h) {
            rustbox.print_char(i, j, rustbox::RB_NORMAL, Color::Black, color,  ' ');
        }
    }
}

fn hollow_rect(x: usize, y: usize, w: usize, h: usize, rustbox: &RustBox, color: Color) {
    // corners
    rustbox.print_char(x, y, rustbox::RB_NORMAL, color, Color::Black,  '┌');
    rustbox.print_char(x+w, y, rustbox::RB_NORMAL, color, Color::Black,  '┐');
    rustbox.print_char(x, y+h, rustbox::RB_NORMAL, color, Color::Black,  '└');
    rustbox.print_char(x+w, y+h, rustbox::RB_NORMAL, color, Color::Black,  '┘');

    // sides
    for i in x+1..(x + w) {
        rustbox.print_char(i, y, rustbox::RB_NORMAL, color, Color::Black,  '─');
        rustbox.print_char(i, y+h, rustbox::RB_NORMAL, color, Color::Black,  '─');
    }// draw initial table
    for i in y+1..(y + h) {
        rustbox.print_char(x, i, rustbox::RB_NORMAL, color, Color::Black,  '│');
        rustbox.print_char(x+w, i, rustbox::RB_NORMAL, color, Color::Black,  '│');
    }

}

fn draw_right_text(y: usize, rustbox: &RustBox, color: Color, text: &str) {
    let text_len = text.len();
    let x = rustbox.width() as usize - text_len;
    rustbox.print(x, y, rustbox::RB_NORMAL, color, Color::Black, text);
}

// draw routines (real)
fn draw_hud(game: &Game, rustbox: &RustBox) {
    if game.has_won {
        rustbox.print(0, 0, rustbox::RB_BOLD, Color::White, Color::Black, format!("You won with {} moves! Press 'r' to reset.", game.moves).as_str());
    } else {
        rustbox.print(0, 0, rustbox::RB_BOLD, Color::White, Color::Black, format!("Moves: {}", game.moves).as_str());
    }
    draw_right_text(0, &rustbox, Color::White, "Press 'q' to quit.");
}

fn draw_table(game: &Game, rustbox: &RustBox) {
    // draw outline of table
    for i in 0..BOARD_SIZE {
        for j in 0..BOARD_SIZE {
            let location = get_block_render_location(game, i, j);
            hollow_rect(location.0, location.1, location.2, location.3, rustbox, Color::White);
        }
    }

    // draw the blocks that are lit :fire:
    for i in 0..BOARD_SIZE {
        for j in 0..BOARD_SIZE {
            if game.board[i][j] {
                let location = get_block_render_location(game, i, j);
                fill_rect(location.0 + 2, location.1 + 1, location.2 - 3, location.3 - 1, rustbox, Color::White);
            }
        }
    }
    
}

fn draw_background(rustbox: &RustBox) {
    let width = rustbox.width();
    let height = rustbox.height();

    for y in 0..height {
        for x in 0..width {
            rustbox.print_char(x, y, rustbox::RB_NORMAL, Color::White, Color::Black, ' ');
        }
    }
}

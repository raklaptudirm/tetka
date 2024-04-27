use std::path::Path;
use std::str::FromStr;

use strum::IntoEnumIterator;

use ataxx::{Board, File, Rank, Square};

use raylib::prelude::*;

const SCALE: f32 = 3.0;
const SQUARE_SIZE: f32 = 27.0;

const WIDTH: i32 = 1280;
const HEIGHT: i32 = 720;

fn main() {
    let board = Board::from_str("x5o/7/7/7/7/7/o5x x 0 1").unwrap();
    let (mut rl, thread) = raylib::init()
        .size(WIDTH, HEIGHT)
        .title("Ataxx GUI")
        .build();

    while !rl.window_should_close() {
        draw_board(&mut rl, &thread, &board).unwrap();
    }
}

fn draw_board(rl: &mut RaylibHandle, thread: &RaylibThread, board: &Board) -> Result<(), String> {
    let squares_txt = load_texture(rl, thread, "ataxx-squares.png")?;
    let pieces_txt = load_texture(rl, thread, "ataxx-pieces.png")?;
    let border_txt = load_texture(rl, thread, "ataxx-board-border.png")?;

    let mut d = rl.begin_drawing(thread);
    d.clear_background(Color::BLACK);

    let top_left = Vector2::new(
        WIDTH as f32 / 2.0 - 183.0 * SCALE / 2.0,
        HEIGHT as f32 / 2.0 - 183.0 * SCALE / 2.0,
    );
    for rank in Rank::iter().rev() {
        for file in File::iter() {
            let square = Square::new(file, rank);
            draw_square(
                &mut d,
                &squares_txt,
                &pieces_txt,
                Vector2::new(
                    top_left.x + file as u32 as f32 * (SQUARE_SIZE - 1.0) * SCALE,
                    top_left.y + rank as u32 as f32 * (SQUARE_SIZE - 1.0) * SCALE,
                ),
                board.at(square),
            )?;
        }
    }

    let src_rec = Rectangle::new(
        0.0,
        0.0,
        border_txt.width() as f32,
        border_txt.height() as f32,
    );
    let dst_rec = Rectangle::new(
        WIDTH as f32 / 2.0 - border_txt.width() as f32 * SCALE / 2.0,
        HEIGHT as f32 / 2.0 - border_txt.height() as f32 * SCALE / 2.0,
        border_txt.width() as f32 * SCALE,
        border_txt.height() as f32 * SCALE,
    );

    d.draw_texture_pro(
        &border_txt,
        src_rec,
        dst_rec,
        Vector2::zero(),
        0.0,
        Color::WHITE,
    );

    Ok(())
}

fn draw_square(
    d: &mut RaylibDrawHandle,
    sq_txt: &Texture2D,
    pc_txt: &Texture2D,
    top_left: Vector2,
    pc: ataxx::Color,
) -> Result<(), String> {
    let a = match pc {
        ataxx::Color::White => 0.0,
        ataxx::Color::Black => 1.0,
        ataxx::Color::None => 0.0, // unused
    };

    let sq_src_rec = Rectangle::new(0.0, 0.0, SQUARE_SIZE, SQUARE_SIZE);
    let pc_src_rec = Rectangle::new(a * SQUARE_SIZE, 0.0, SQUARE_SIZE, SQUARE_SIZE);

    let dst_rec = Rectangle::new(
        top_left.x,
        top_left.y,
        SQUARE_SIZE * SCALE,
        SQUARE_SIZE * SCALE,
    );

    d.draw_texture_pro(
        sq_txt,
        sq_src_rec,
        dst_rec,
        Vector2::new(0.0, 0.0),
        0.0,
        Color::WHITE,
    );

    if pc == ataxx::Color::None {
        return Ok(());
    }

    d.draw_texture_pro(
        pc_txt,
        pc_src_rec,
        dst_rec,
        Vector2::new(0.0, 0.0),
        0.0,
        Color::WHITE,
    );

    Ok(())
}

fn load_texture(
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
    name: &str,
) -> Result<Texture2D, String> {
    let path = Path::new(file!())
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
    let img = Image::load_image(&(path + "/assets/" + name))?;
    rl.load_texture_from_image(thread, &img)
}

use std::str::FromStr;

use ataxx::{Board, File, Rank, Square};
use raylib::prelude::*;

use strum::IntoEnumIterator;

pub struct GuiBuilder {
    title: String,
    fen: String,
    width: i32,
    height: i32,
    scale: f32,
}

impl GuiBuilder {
    pub fn new() -> GuiBuilder {
        GuiBuilder {
            title: "Ataxx GUI".to_string(),
            fen: "x5o/7/7/7/7/7/o5x x 0 1".to_string(),
            width: 1280,
            height: 720,
            scale: 3.0,
        }
    }

    pub fn build(&self) -> Gui {
        let (mut raylib, thread) = raylib::init()
            .size(self.width, self.height)
            .title(&self.title)
            .build();
        let textures = Textures::load(&mut raylib, &thread);
        let board = Board::from_str(&self.fen).unwrap();

        Gui {
            raylib,
            thread,
            options: GuiOptions {
                width: self.width,
                height: self.height,
                scale: self.scale,
                board,
                textures,
            },
        }
    }
}

pub struct Gui {
    raylib: RaylibHandle,
    thread: RaylibThread,

    options: GuiOptions,
}

impl Gui {
    pub fn new_drawer(&mut self) -> Drawer {
        Drawer {
            gui: &self.options,
            drawer: self.raylib.begin_drawing(&self.thread),
        }
    }

    pub fn should_quit(&self) -> bool {
        self.raylib.window_should_close()
    }
}

struct GuiOptions {
    width: i32,
    height: i32,

    scale: f32,

    board: ataxx::Board,

    textures: Textures,
}

pub struct Drawer<'a> {
    gui: &'a GuiOptions,
    drawer: RaylibDrawHandle<'a>,
}

const SQUARE_SIZE: f32 = 27.0;

impl<'a> Drawer<'a> {
    pub fn draw(&mut self) {
        self.draw_board();
    }

    fn draw_board(&mut self) {
        self.drawer.clear_background(Color::BLACK);

        for rank in Rank::iter().rev() {
            for file in File::iter() {
                self.draw_square(Square::new(file, rank));
            }
        }

        let src_rec = Rectangle::new(
            0.0,
            0.0,
            self.gui.textures.border.width() as f32,
            self.gui.textures.border.height() as f32,
        );
        let dst_rec = Rectangle::new(
            self.gui.width as f32 / 2.0
                - self.gui.textures.border.width() as f32 * self.gui.scale / 2.0,
            self.gui.height as f32 / 2.0
                - self.gui.textures.border.height() as f32 * self.gui.scale / 2.0,
            self.gui.textures.border.width() as f32 * self.gui.scale,
            self.gui.textures.border.height() as f32 * self.gui.scale,
        );

        self.drawer.draw_texture_pro(
            &self.gui.textures.border,
            src_rec,
            dst_rec,
            Vector2::zero(),
            0.0,
            Color::WHITE,
        );
    }

    fn draw_square(&mut self, sq: Square) {
        let pc = self.gui.board.at(sq);
        let a = match pc {
            ataxx::Color::White => 0.0,
            ataxx::Color::Black => 1.0,
            ataxx::Color::None => 0.0, // unused
        };

        let top_left = Vector2::new(
            self.gui.width as f32 / 2.0 - 183.0 * self.gui.scale / 2.0
                + sq.file() as u32 as f32 * (SQUARE_SIZE - 1.0) * self.gui.scale,
            self.gui.height as f32 / 2.0 - 183.0 * self.gui.scale / 2.0
                + sq.rank() as u32 as f32 * (SQUARE_SIZE - 1.0) * self.gui.scale,
        );

        let sq_src_rec = Rectangle::new(0.0, 0.0, SQUARE_SIZE, SQUARE_SIZE);
        let pc_src_rec = Rectangle::new(a * SQUARE_SIZE, 0.0, SQUARE_SIZE, SQUARE_SIZE);

        let dst_rec = Rectangle::new(
            top_left.x,
            top_left.y,
            SQUARE_SIZE * self.gui.scale,
            SQUARE_SIZE * self.gui.scale,
        );

        self.drawer.draw_texture_pro(
            &self.gui.textures.square,
            sq_src_rec,
            dst_rec,
            Vector2::new(0.0, 0.0),
            0.0,
            Color::WHITE,
        );

        if pc == ataxx::Color::None {
            // No piece to draw.
            return;
        }

        self.drawer.draw_texture_pro(
            &self.gui.textures.pieces,
            pc_src_rec,
            dst_rec,
            Vector2::new(0.0, 0.0),
            0.0,
            Color::WHITE,
        );
    }
}

struct Textures {
    pieces: Texture2D,
    square: Texture2D,
    border: Texture2D,
}

macro_rules! load_png_texture {
    ($rl:expr, $thread:expr, $file:expr) => {{
        let raw = include_bytes!($file);
        let img = Image::load_image_from_mem(".png", &raw.to_vec(), raw.len() as i32).unwrap();
        $rl.load_texture_from_image($thread, &img).unwrap()
    }};
}

impl Textures {
    fn load(rl: &mut RaylibHandle, thread: &RaylibThread) -> Textures {
        Textures {
            pieces: load_png_texture!(rl, thread, "../assets/ataxx-pieces.png"),
            square: load_png_texture!(rl, thread, "../assets/ataxx-square.png"),
            border: load_png_texture!(rl, thread, "../assets/ataxx-border.png"),
        }
    }
}

use arrayvec::ArrayVec;
use chess::{Game, Piece, ALL_PIECES, ALL_SQUARES};
use tetra::graphics::mesh::{GeometryBuilder, Mesh, ShapeStyle};
use tetra::graphics::{Color, DrawParams, Rectangle, Texture};
use tetra::input::MouseButton;
use tetra::math::Vec2;
use tetra::{graphics, input, Context, ContextBuilder, Event, State};
use crate::{System, GameInfo};

pub const SCALE: f32 = 90.0;
const SCALE_US: usize = SCALE as usize;
const TEX_SIZE: f32 = 360.0;
const TEX_SCALE: f32 = SCALE / TEX_SIZE;

const DARK_BG_COLOR: Color = Color::rgb(94.0 / 255.0, 118.0 / 255.0, 136.0 / 255.0);
const LIGHT_BG_COLOR: Color = Color::rgb(135.0 / 255.0, 162.0 / 255.0, 183.0 / 255.0);
const SELECT_COLOR: Color = Color::rgb(76.0 / 255.0, 123.0 / 255.0, 105.0 / 255.0);
const LAST_MOVE_COLOR: Color = Color::rgb(143.0 / 255.0, 178.0 / 255.0, 108.0 / 255.0);

pub struct Graphics {
    pieces: [Texture; 12],
    draw_for: usize,
}

impl System {
    pub fn start(game: Game) {
        ContextBuilder::new("chessie", (8.0 * SCALE) as i32, (8.0 * SCALE) as i32)
            .quit_on_escape(true)
            .show_mouse(true)
            .build()
            .unwrap()
            .run(|mut ctx| {
                Ok(Self {
                    game,
                    gui: Graphics {
                        pieces: make_textures(&mut ctx),
                        draw_for: 5,
                    },
                    info: GameInfo::default()
                })
            })
            .unwrap()
    }

    fn get_piece(&self, color: chess::Color, piece: Piece) -> &Texture {
        let idx = piece.to_index() + color.to_index() * 6;
        &self.gui.pieces[idx]
    }
}

impl State for System {
    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        if self.gui.draw_for == 0 {
            return Ok(());
        }

        graphics::clear(ctx, DARK_BG_COLOR);

        // Draw background
        let mut builder = GeometryBuilder::new();
        for idx in 0..64 {
            let y = idx / 8;
            if (idx % 2) != (y % 2) {
                builder.rectangle(ShapeStyle::Fill, rect_from_square(idx))?;
            }
        }
        let mesh = builder.build_mesh(ctx)?;
        mesh.draw(ctx, DrawParams::new().color(LIGHT_BG_COLOR));

        // Draw last move
        if let Some(mov) = self.info.last_move {
            let mesh = Mesh::rectangle(ctx, ShapeStyle::Fill, rect_from_square(mov.get_source().to_index()))?;
            mesh.draw(ctx, DrawParams::new().color(LAST_MOVE_COLOR));
            let mesh = Mesh::rectangle(ctx, ShapeStyle::Fill, rect_from_square(mov.get_dest().to_index()))?;
            mesh.draw(ctx, DrawParams::new().color(LAST_MOVE_COLOR));
        }

        // Draw selection
        if let Some(square) = self.info.selected_square {
            let mesh = Mesh::rectangle(ctx, ShapeStyle::Fill, rect_from_square(square))?;
            mesh.draw(ctx, DrawParams::new().color(SELECT_COLOR));

            // Draw possible moves
            let mut builder = GeometryBuilder::new();
            for mov in self.possible_moves().filter(|mov| mov.get_source().to_index() == square) {
                let idx = mov.get_dest().to_index();
                builder.circle(ShapeStyle::Fill, circle_from_square(idx), SCALE / 7.0)?;
            }
            let mesh = builder.build_mesh(ctx)?;
            mesh.draw(ctx, DrawParams::new().color(SELECT_COLOR));
        }

        // Draw red king if in check
        let board = self.game.current_position();
        if board.checkers().0 != 0 {
            let square = board.king_square(board.side_to_move()).to_index();
            let mesh = Mesh::circle(ctx, ShapeStyle::Fill, circle_from_square(square), SCALE / 2.5)?;
            mesh.draw(ctx, DrawParams::new().color(Color::RED.with_alpha(0.6)));
        }

        // Draw pieces
        for square in ALL_SQUARES {
            let piece = board.piece_on(square);
            if let Some(piece) = piece {
                let color = board.color_on(square).unwrap();

                let tex = self.get_piece(color, piece);
                let params = DrawParams::new()
                    .position(pos_from_square(square.to_index()))
                    .scale(Vec2::new(TEX_SCALE, TEX_SCALE));
                tex.draw(ctx, params);
            }
        }

        self.gui.draw_for -= 1;
        Ok(())
    }

    fn event(&mut self, ctx: &mut Context, event: Event) -> tetra::Result {
        if let Event::MouseButtonPressed {
            button: MouseButton::Left,
        } = event
        {
            let pos = input::get_mouse_position(ctx).round();
            let (x, y) = (pos.x as usize, pos.y as usize);
            let square = ((7 - (y / SCALE_US)) * 8) + (x / SCALE_US);
            self.square_clicked(square);
        }

        self.gui.draw_for = 5;
        Ok(())
    }
}

fn make_textures(ctx: &mut Context) -> [Texture; 12] {
    let mut textures = ArrayVec::new();
    let mut load = |color: chess::Color| {
        for piece in ALL_PIECES {
            let name = format!("./resources/pieces/{}.png", piece.to_string(color));
            textures.push(Texture::new(ctx, name).unwrap());
        }
    };
    load(chess::Color::White);
    load(chess::Color::Black);
    textures.into_inner().unwrap()
}

fn pos_from_square(square: usize) -> Vec2<f32> {
    let x = square % 8;
    let y = 7 - (square / 8);
    Vec2::new(x as f32 * SCALE, y as f32 * SCALE)
}

fn rect_from_square(square: usize) -> Rectangle {
    let x = square % 8;
    let y = 7 - (square / 8);
    Rectangle::new(x as f32 * SCALE, y as f32 * SCALE, SCALE, SCALE)
}

fn circle_from_square(square: usize) -> Vec2<f32> {
    pos_from_square(square) + (SCALE / 2.0)
}

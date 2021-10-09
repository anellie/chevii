use arrayvec::ArrayVec;
use chess::{Game, Piece, ALL_PIECES, ALL_SQUARES};
use tetra::graphics::mesh::{GeometryBuilder, ShapeStyle};
use tetra::graphics::{Color, DrawParams, Rectangle, Texture};
use tetra::math::Vec2;
use tetra::{graphics, Context, ContextBuilder, Event, State};
use tetra::input::MouseButton;

pub const SCALE: f32 = 90.0;
const TEX_SIZE: f32 = 360.0;
const TEX_SCALE: f32 = SCALE / TEX_SIZE;

pub struct System {
    pub game: Game,
    pieces: [Texture; 12],
    selected_square_idx: Option<usize>,
    needs_draw: bool,
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
                    pieces: Self::make_textures(&mut ctx),
                    selected_square_idx: None,
                    needs_draw: true,
                })
            })
            .unwrap()
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

    fn get_piece(&self, color: chess::Color, piece: Piece) -> &Texture {
        let idx = piece.to_index() + color.to_index() * 6;
        &self.pieces[idx]
    }

    fn mouse_click(&mut self, ctx: &mut Context) {
        todo!();
    }
}

impl State for System {
    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        if !self.needs_draw {
            return Ok(());
        }

        let board = self.game.current_position();
        graphics::clear(ctx, Color::rgb(94.0 / 255.0, 118.0 / 255.0, 136.0 / 255.0));

        let mut builder = GeometryBuilder::new();
        let style = ShapeStyle::Fill;
        let color = Color::rgb(135.0 / 255.0, 162.0 / 255.0, 183.0 / 255.0);

        for idx in 0..64 {
            let y = idx / 8;
            if (idx % 2) == (y % 2) {
                let x = idx % 8;
                let bounds = Rectangle::new(x as f32 * SCALE, y as f32 * SCALE, SCALE, SCALE);
                builder.rectangle(style, bounds)?;
            }
        }

        let mesh = builder.build_mesh(ctx)?;
        mesh.draw(
            ctx,
            DrawParams::new().position(Vec2::new(0.0, 0.0)).color(color),
        );

        for square in ALL_SQUARES {
            let piece = board.piece_on(square);
            if let Some(piece) = piece {
                let color = board.color_on(square).unwrap();
                let idx = square.to_index();
                let x = idx % 8;
                let y = 7 - (idx / 8);

                let tex = self.get_piece(color, piece);
                let params = DrawParams::new()
                    .position(Vec2::new(x as f32 * SCALE, y as f32 * SCALE))
                    .scale(Vec2::new(TEX_SCALE, TEX_SCALE));
                tex.draw(ctx, params);
            }
        }

        self.needs_draw = false;
        Ok(())
    }

    fn event(&mut self, ctx: &mut Context, event: Event) -> tetra::Result {
        if let Event::MouseButtonPressed { button: MouseButton::Left } = event {
            self.mouse_click(ctx);
        }

        self.needs_draw = true;
        Ok(())
    }
}

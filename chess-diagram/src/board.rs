// use crate::pieces::Piece;
// use Piece::*;
use egui::{
    Align2, Color32, Context, CornerRadius, FontId, Pos2, Rect, Stroke, StrokeKind, Ui, Vec2, pos2,
    vec2,
};
use shakmaty::{Chess, File, Move, Position, Rank, Square};

use crate::{gesture::Gesture, sources::Sources};

const LIGHT_SQUARE: bool = true;
const DARK_SQUARE: bool = false;

const MARGIN: f32 = 64.0;

#[rustfmt::skip]
const BOARD_COLORS: [[bool;8];8] = [
    [LIGHT_SQUARE, DARK_SQUARE,LIGHT_SQUARE, DARK_SQUARE,LIGHT_SQUARE, DARK_SQUARE,LIGHT_SQUARE, DARK_SQUARE],
    [DARK_SQUARE,LIGHT_SQUARE, DARK_SQUARE,LIGHT_SQUARE, DARK_SQUARE,LIGHT_SQUARE, DARK_SQUARE, LIGHT_SQUARE],
    [LIGHT_SQUARE, DARK_SQUARE,LIGHT_SQUARE, DARK_SQUARE,LIGHT_SQUARE, DARK_SQUARE,LIGHT_SQUARE, DARK_SQUARE],
    [DARK_SQUARE,LIGHT_SQUARE, DARK_SQUARE,LIGHT_SQUARE, DARK_SQUARE,LIGHT_SQUARE, DARK_SQUARE, LIGHT_SQUARE],
    [LIGHT_SQUARE, DARK_SQUARE,LIGHT_SQUARE, DARK_SQUARE,LIGHT_SQUARE, DARK_SQUARE,LIGHT_SQUARE, DARK_SQUARE],
    [DARK_SQUARE,LIGHT_SQUARE, DARK_SQUARE,LIGHT_SQUARE, DARK_SQUARE,LIGHT_SQUARE, DARK_SQUARE, LIGHT_SQUARE],
    [LIGHT_SQUARE, DARK_SQUARE,LIGHT_SQUARE, DARK_SQUARE,LIGHT_SQUARE, DARK_SQUARE,LIGHT_SQUARE, DARK_SQUARE],
    [DARK_SQUARE,LIGHT_SQUARE, DARK_SQUARE,LIGHT_SQUARE, DARK_SQUARE,LIGHT_SQUARE, DARK_SQUARE, LIGHT_SQUARE],
];

pub fn render_board(
    ctx: &Context,
    ui: &Ui,
    sources: &Sources<'_>,
    gesture: &Gesture,
    game: &Chess,
    last_move: Option<&Move>,
    title: Option<String>,
    highlight_square: Option<Square>,
) {
    // let mut state = ss_main.borrow_mut();
    let lid = ui.layer_id();
    // let rect = ctx.screen_rect();
    let rect = ui.max_rect();
    let painter = egui::Painter::new(ctx.clone(), lid, rect);
    let board_rect = board_rect(&rect);
    let square_size = board_rect.width() / 8.0;

    // backround
    let _ = painter.rect_filled(rect, CornerRadius::ZERO, Color32::WHITE);

    // title
    title.map(|title| {
        let pos = pos2(
            board_rect.min.x + board_rect.width() / 2.0,
            rect.min.y + MARGIN / 3.0,
        );
        let align = Align2::CENTER_TOP;
        let font = FontId::proportional(16.0);
        let color = Color32::BLACK;
        let _ = painter.text(pos, align, title, font, color);
    });

    // frame
    let _ = painter.rect_stroke(
        board_rect.expand(6.0),
        CornerRadius::same(1),
        (6.0, Color32::BLACK),
        StrokeKind::Middle,
    );

    // turn
    let turn_margin_x = MARGIN * 0.7;
    let turn_margin_y = MARGIN / 3.0;
    let turn_radius = 12.0;
    if game.turn() == shakmaty::Color::Black {
        let top = board_rect.min.y + turn_margin_y;
        let left = board_rect.max.x + turn_margin_x;
        let _ = painter.circle(pos2(left, top), turn_radius, Color32::BLACK, Stroke::NONE);
    } else {
        let top = board_rect.max.y - turn_margin_y;
        let left = board_rect.max.x + turn_margin_x;
        let stroke_width = 3.0;
        // let radius = turn_radius - (stroke_width / 2.0);
        let radius = turn_radius - stroke_width;
        let _ = painter.circle(
            pos2(left, top),
            radius,
            Color32::WHITE,
            (stroke_width, Color32::BLACK),
        );
    }

    // let last_move_from = last_move.and_then(|move_| move_.from());
    let last_move_to = last_move.map(|move_| move_.to());

    for rank_index in 0..8usize {
        for file_index in 0..8usize {
            let top = rank_index as f32 * square_size + board_rect.min.y;
            let left = file_index as f32 * square_size + board_rect.min.x;
            let color = BOARD_COLORS[rank_index][file_index];
            let square_rect =
                Rect::from_two_pos(pos2(left, top), pos2(left + square_size, top + square_size));
            if color {
                let _ = painter.rect_filled(square_rect, CornerRadius::ZERO, Color32::WHITE);
            } else {
                let _ = sources.get("dark-square").map(|dark_square| {
                    dark_square.paint_at(ui, square_rect);
                });
            }

            let rank = rank_from_index(rank_index);
            let file = file_from_index(file_index);
            let square = Square::from_coords(file, rank);

            let (text_color, font, rank_text, file_text) = {
                if let Some(to) = last_move_to
                    && square == to
                {
                    (
                        Color32::BLACK,
                        FontId::proportional(24.0),
                        format!("{rank} ·"),
                        format!("  {file} ·"),
                    )
                } else {
                    (
                        Color32::BLACK,
                        FontId::proportional(24.0),
                        format!("{rank}  "),
                        format!("  {file}  "),
                    )
                }
            };

            // TODO files and ranks
            let rank_x = board_rect.min.x - (MARGIN / 2.0);
            let rank_y = top + square_size;
            // let _ = painter.rect_filled(
            //     Rect::from_min_size(pos2(rank_x, top), vec2(MARGIN / 2.0, square_size)),
            //     CornerRadius::ZERO,
            //     Color32::WHITE,
            // );
            let _ = painter.text(
                pos2(rank_x, rank_y),
                Align2::CENTER_BOTTOM,
                rank_text,
                font.clone(),
                text_color,
            );
            let file_y = board_rect.max.y + (MARGIN * 0.6);
            let file_x = left + (square_size / 2.0);
            // let _ = painter.rect_filled(
            //     Rect::from_min_size(
            //         pos2(file_x, y_offset + board_size + 10.0),
            //         vec2(square_size, MARGIN),
            //     ),
            //     CornerRadius::ZERO,
            //     Color32::WHITE,
            // );
            let _ = painter.text(
                pos2(file_x, file_y),
                Align2::CENTER_BOTTOM,
                file_text,
                font.clone(),
                text_color,
            );

            highlight_square.map(|highlight| {
                if highlight == square {
                    let _ = painter.rect_filled(
                        square_rect,
                        CornerRadius::same(2),
                        Color32::from_rgba_unmultiplied(0, 183, 235, 24),
                    );
                }
            });

            // match (last_move_from, last_move_to) {
            //     (Some(_from), Some(to)) => {
            //         let square = Square::from_coords(file, rank);

            //         if square == to {
            //             let _ = painter.rect_filled(
            //                 square_rect,
            //                 CornerRadius::ZERO,
            //                 Color32::from_rgba_unmultiplied(0, 200, 0, 8),
            //             );
            //         };
            //     }
            //     _ => {}
            // }

            if let Some(piece) = game.board().piece_at(square) {
                let piece_name = format!("{}", piece.char());
                if let Some(image) = sources.get(piece_name) {
                    image.paint_at(ui, square_rect);
                }
            }
        }
    }

    // for rank_index in 0..8usize {
    //     for file_index in 0..8usize {
    //         match (last_move_from, last_move_to) {
    //             (Some(from), Some(to)) => {
    //                 let top = rank_index as f32 * square_size + y_offset;
    //                 let left = file_index as f32 * square_size + x_offset;
    //                 let square_rect = Rect::from_two_pos(
    //                     pos2(left, top),
    //                     pos2(left + square_size, top + square_size),
    //                 );
    //                 let rank = rank_from_index(rank_index);
    //                 let file = file_from_index(file_index);
    //                 let square = Square::from_coords(file, rank);

    //                 if square == from {
    //                     let _ = painter.rect_stroke(
    //                         square_rect,
    //                         CornerRadius::ZERO,
    //                         (1.0, Color32::GRAY),
    //                         StrokeKind::Inside,
    //                     );
    //                 };
    //                 if square == to {
    //                     let _ = painter.rect_stroke(
    //                         square_rect,
    //                         CornerRadius::ZERO,
    //                         (1.0, Color32::GRAY),
    //                         StrokeKind::Middle,
    //                     );
    //                 };
    //             }
    //             _ => {}
    //         }
    //     }
    // }

    if let Gesture::Moving(state) = gesture {
        let piece_name = format!("{}", state.piece().char());
        let pos = state.position();

        if let Some(image) = sources.get(piece_name) {
            let rect = Rect::from_center_size(pos, Vec2::new(square_size, square_size));
            image.paint_at(ui, rect);
        }
    }
}

pub fn board_rect(rect: &Rect) -> Rect {
    let whole = if rect.width() > rect.height() {
        let sz = rect.height();
        let left = rect.min.x + ((rect.width() - sz) / 2.0);
        let top = rect.min.y;

        Rect::from_min_size(pos2(left, top), vec2(sz, sz))
    } else {
        let sz = rect.width();
        let left = rect.min.x;
        let top = rect.min.y + ((rect.height() - sz) / 2.0);

        Rect::from_min_size(pos2(left, top), vec2(sz, sz))
    };

    whole.expand(-MARGIN)
}

pub fn rank_from_index(index: usize) -> Rank {
    match index {
        0 => Rank::Eighth,
        1 => Rank::Seventh,
        2 => Rank::Sixth,
        3 => Rank::Fifth,
        4 => Rank::Fourth,
        5 => Rank::Third,
        6 => Rank::Second,
        7 => Rank::First,
        _ => panic!("rank out of range {index}"),
    }
}

pub fn file_from_index(index: usize) -> File {
    match index {
        0 => File::A,
        1 => File::B,
        2 => File::C,
        3 => File::D,
        4 => File::E,
        5 => File::F,
        6 => File::G,
        7 => File::H,
        _ => panic!("file out of range {index}"),
    }
}
pub fn rank_to_index(rank: Rank) -> usize {
    match rank {
        Rank::Eighth => 0,
        Rank::Seventh => 1,
        Rank::Sixth => 2,
        Rank::Fifth => 3,
        Rank::Fourth => 4,
        Rank::Third => 5,
        Rank::Second => 6,
        Rank::First => 7,
    }
}

pub fn file_to_index(file: File) -> usize {
    match file {
        File::A => 0,
        File::B => 1,
        File::C => 2,
        File::D => 3,
        File::E => 4,
        File::F => 5,
        File::G => 6,
        File::H => 7,
    }
}

pub fn square_at(container_rect: &Rect, pos: Pos2) -> Option<Square> {
    let board_rect = board_rect(container_rect);
    if board_rect.contains(pos) {
        let x = pos.x - board_rect.min.x;
        let y = pos.y - board_rect.min.y;

        let board_x = (x * 8.0 / board_rect.width()).floor();
        let board_y = (y * 8.0 / board_rect.height()).floor();

        let file = file_from_index(board_x as usize);
        let rank = rank_from_index(board_y as usize);

        Some(Square::from_coords(file, rank))
    } else {
        None
    }
}

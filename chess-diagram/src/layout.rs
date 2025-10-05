use std::num::NonZero;

use egui::{
    Align, Color32, FontSelection, Pos2, Rect, RichText, Sense, Style, Ui, Vec2, pos2,
    text::LayoutJob, vec2,
};

use crate::variation::MoveIndex;

struct State {
    pos: Pos2,
    line_height: f32,
    var_depth: usize,
}

impl State {
    fn new(pos: Pos2) -> Self {
        Self {
            pos,
            line_height: 32.0,
            var_depth: 0,
        }
    }
}

pub struct ScoreSheetMove {
    san: RichText,
    index: Option<MoveIndex>,
}

impl ScoreSheetMove {
    pub fn new(san: RichText, index: Option<MoveIndex>) -> Self {
        Self { san, index }
    }
}

impl From<(RichText, Option<MoveIndex>)> for ScoreSheetMove {
    fn from(value: (RichText, Option<MoveIndex>)) -> Self {
        ScoreSheetMove::new(value.0, value.1)
    }
}

pub enum ScoreSheetItem {
    MainVariationMove {
        ord: RichText,
        white: ScoreSheetMove,
        black: ScoreSheetMove,
    },
    VariationMove {
        ord: RichText,
        white: ScoreSheetMove,
        black: ScoreSheetMove,
    },
    StartVariation,
    EndVariation,
}

pub fn main_variation_move(
    ord: RichText,
    white: ScoreSheetMove,
    black: ScoreSheetMove,
) -> ScoreSheetItem {
    ScoreSheetItem::MainVariationMove { ord, white, black }
}

pub fn variation_move(
    ord: RichText,
    white: ScoreSheetMove,
    black: ScoreSheetMove,
) -> ScoreSheetItem {
    ScoreSheetItem::VariationMove { ord, white, black }
}

pub fn start_variation() -> ScoreSheetItem {
    ScoreSheetItem::StartVariation
}

pub fn end_variation() -> ScoreSheetItem {
    ScoreSheetItem::EndVariation
}

fn richtext_to_layout(rt: RichText) -> LayoutJob {
    let style = Style::default();
    let mut layout_job = LayoutJob::default();
    rt.append_to(
        &mut layout_job,
        &style,
        FontSelection::Default,
        Align::Center,
    );
    layout_job
}

pub struct ScoreSheet {
    state: State,
    rect: Rect,
}

impl ScoreSheet {
    pub fn new(rect: Rect) -> Self {
        Self {
            rect,
            state: State::new(rect.min),
        }
    }
}

impl ScoreSheet {
    fn project_text(&self, text: &str) -> Rect {
        // we do a very very rough estimate, it's the begining
        let text_len = text.len() as f32;
        let line_y = self.state.line_height;
        let bottom_right = pos2(self.state.pos.x + text_len * 6.0, self.state.pos.y + line_y);
        Rect::from_two_pos(self.state.pos, bottom_right)
    }

    fn text_size(&self, text: &str) -> Vec2 {
        let text_len = text.len() as f32;
        vec2(
            text_len * self.state.line_height * 0.6,
            self.state.line_height,
        )
    }

    fn new_line(&mut self) {
        self.state.pos.x = self.rect.min.x;
        self.state.pos.y += self.state.line_height;
    }

    pub fn push_item(&mut self, ui: &mut Ui, item: ScoreSheetItem) -> Option<MoveIndex> {
        use ScoreSheetItem::*;
        match item {
            StartVariation => self.push_start_variation(ui),
            EndVariation => self.push_end_variation(ui),
            MainVariationMove { ord, white, black } => {
                self.push_main_variation_move(ui, ord, white, black)
            }
            VariationMove { ord, white, black } => self.push_variation_move(ui, ord, white, black),
        }
    }
    /// layout a move from main variation
    /// | left-margin ord. space white-move middle black-move |    
    fn push_main_variation_move(
        &mut self,
        ui: &mut Ui,
        ord: RichText,
        white: ScoreSheetMove,
        black: ScoreSheetMove,
    ) -> Option<MoveIndex> {
        self.new_line();
        log::info!("push_main_variation_move {}", self.state.pos);

        let ord_size = self.text_size(ord.text());
        let ord_response = ui.allocate_rect(
            Rect::from_min_size(self.state.pos, ord_size),
            Sense::click(),
        );
        let white_response = ui.allocate_rect(
            Rect::from_min_size(
                pos2(self.state.pos.x + ord_size.x, self.state.pos.y),
                self.text_size(white.san.text()),
            ),
            Sense::click(),
        );
        let black_response = ui.allocate_rect(
            Rect::from_min_max(
                pos2(
                    self.state.pos.x + (self.rect.width() / 2.0),
                    self.state.pos.y,
                ),
                pos2(self.rect.max.x, self.state.pos.y + self.state.line_height),
            ),
            Sense::click(),
        );
        log::info!("ord {}", ord_response.rect);
        log::info!("white {}", white_response.rect);
        log::info!("black {}", black_response.rect);

        let painter = ui.painter();
        // painter.layout_job(layout_job);
        let fonts = ui.fonts(|f| f.clone());
        let ord_galley = fonts.layout_job(richtext_to_layout(ord));
        let white_galley = fonts.layout_job(richtext_to_layout(white.san));
        let black_galley = fonts.layout_job(richtext_to_layout(black.san));
        painter.galley(ord_response.rect.min, ord_galley, Color32::BLACK);
        painter.galley(white_response.rect.min, white_galley, Color32::BLACK);
        painter.galley(black_response.rect.min, black_galley, Color32::BLACK);

        if white_response.clicked() {
            white.index
        } else if black_response.clicked() {
            black.index
        } else {
            None
        }
    }

    fn push_variation_move(
        &mut self,
        ui: &mut Ui,
        ord: RichText,
        white: ScoreSheetMove,
        black: ScoreSheetMove,
    ) -> Option<MoveIndex> {
        None
    }

    fn push_start_variation(&mut self, ui: &mut Ui) -> Option<MoveIndex> {
        self.new_line();
        None
    }

    fn push_end_variation(&mut self, ui: &mut Ui) -> Option<MoveIndex> {
        self.new_line();
        None
    }
}

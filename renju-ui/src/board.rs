use egui::{style::Margin, *};
use renju::{
    board::{Board, MoveIndex},
    board_logic::{BoardArr, BoardMarker, Point, Stone},
    evaluator,
    file_reader::renlib::{CommandError, CommandVariant},
    p,
};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct UIBoard {
    board: BoardArr,
    moves: Vec<Point>,
    graph: Board,
    variants: Vec<(BoardMarker, MoveIndex)>,
}

impl UIBoard {
    pub fn new() -> Self {
        Self {
            board: BoardArr::new(15),
            moves: vec![],
            graph: Board::new(),
            variants: vec![],
        }
    }

    pub fn moves(&self) -> &[Point] {
        self.moves.as_ref()
    }

    pub fn board(&self) -> &BoardArr {
        &self.board
    }

    pub fn graph(&self) -> &Board {
        &self.graph
    }

    /// Take care to ensure you call update after doing your stuff.
    pub fn graph_mut(&mut self) -> &mut Board {
        &mut self.graph
    }
}

impl Default for UIBoard {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy)]
struct BoardRender {
    rect: Rect,
    lines: u16,
    lines_f: f32,
    incr: f32,
    x_offset: f32,
    y_offset: f32,
    sq_size: f32,
}

impl BoardRender {
    fn lines(&self, painter: &Painter) {
        let BoardRender {
            rect,
            lines,
            lines_f,
            incr,
            x_offset,
            y_offset,
            sq_size,
            ..
        } = *self;

        let x_range = (x_offset + rect.left())..=(x_offset + rect.left() + incr * (lines_f - 1.0));
        let y_range = (y_offset + rect.top())..=(y_offset + rect.top() + incr * (lines_f - 1.0));
        for x_i in 0..lines {
            let x_i = f32::from(x_i);
            painter.vline(
                x_offset + rect.left() + incr * x_i,
                y_range.clone(),
                Stroke::new(1.0, Color32::BLACK),
            );
            for y_i in 0..lines {
                let y_i = f32::from(y_i);

                painter.hline(
                    x_range.clone(),
                    y_offset + rect.top() + incr * y_i,
                    Stroke::new(1.0, Color32::BLACK),
                );
            }
        }
        // paint guide squares
        for point in [p![H, 8], p![D, 12], p![D, 4], p![L, 4], p![L, 12]] {
            painter.rect_filled(
                Rect::from_center_size(self.pos_at(&point), Vec2::splat(sq_size * 0.01)),
                Rounding::none(),
                Color32::BLACK,
            );
        }
    }

    fn pos_at(&self, point: &renju::board_logic::Point) -> Pos2 {
        let BoardRender {
            rect,
            incr,
            x_offset,
            y_offset,
            ..
        } = *self;
        Pos2::new(
            x_offset + rect.left() + incr * point.x as f32,
            y_offset + rect.top() + incr * point.y as f32,
        )
    }

    fn marks(&self, painter: &Painter, board: &UIBoard) {
        let BoardRender { lines, sq_size, .. } = *self;

        let children = board.graph.get_children(&board.graph.current_move());
        // find other trees with same outcome if placed
        for (m, _) in &board.variants {
            if board.board.get_point(m.point).unwrap().color.is_empty() && m.command.is_move() {
                let pos = self.pos_at(&m.point);
                painter.circle(pos, 3.0, Color32::GOLD, Stroke::new(2.0, Color32::BLACK))
            }
        }
        for child in children {
            let marker = board.graph.get_move(child).unwrap();
            let pos = self.pos_at(&marker.point);
            painter.circle(pos, 3.0, Color32::WHITE, Stroke::new(2.0, Color32::BLACK))
        }
    }

    fn closest(&self, pos: &Pos2) -> Option<Point> {
        let BoardRender {
            rect,
            incr,
            x_offset,
            y_offset,
            lines,
            ..
        } = *self;

        let x = pos.x - x_offset - rect.left() + incr / 2.0;
        let y = pos.y - y_offset - rect.top() + incr / 2.0;
        if let (x_div @ 0.., y_div @ 0..) = (x.div_euclid(incr) as i32, y.div_euclid(incr) as i32) {
            if x_div >= lines as i32 || y_div >= lines as i32 {
                return None;
            }
            let point = Point::new(x_div as u32, y_div as u32);
            let real_pos = self.pos_at(&point);
            if real_pos.distance(*pos) <= (incr / 2.4) {
                Some(point)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn stones(&self, painter: &Painter, board: &UIBoard) {
        for stone in board.board.iter() {
            let pos = self.pos_at(&stone.point);
            if stone.color.is_empty() || stone.command.is_no_move() {
                if stone.command.is_mark() {
                    painter.text(
                        pos,
                        Align2::CENTER_CENTER,
                        stone
                            .oneline_comment
                            .as_deref()
                            .unwrap_or_default()
                            .to_string(),
                        FontId::monospace(14.0),
                        painter.ctx().style().visuals.text_color(),
                    );
                }
            } else {
                painter.circle(
                    pos,
                    self.incr / 2.0,
                    if stone.color == Stone::Black {
                        Color32::BLACK
                    } else if stone.color == Stone::White {
                        Color32::WHITE
                    } else {
                        unimplemented!()
                    },
                    (1.0, Color32::DARK_GRAY),
                );
                let num_move = board.moves.iter().position(|p| p == &stone.point).unwrap() + 1;
                painter.text(
                    pos,
                    Align2::CENTER_CENTER,
                    format!("{num_move}"),
                    FontId::monospace(14.0),
                    if stone.color == Stone::Black {
                        Color32::WHITE
                    } else if stone.color == Stone::White {
                        Color32::BLACK
                    } else {
                        unimplemented!()
                    },
                );
            }
        }
    }
}

impl UIBoard {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let size = ui.available_size();
        ui.add_sized(size, |ui: &mut Ui| {
            egui::Frame::default()
                .inner_margin(Margin::same(0.0))
                .show(ui, |ui| {
                    let (response, painter) = ui.allocate_painter(size, Sense::click());
                    let lines = self.board.size() as u16;
                    let lines_f = self.board.size() as f32;
                    let mut max_rect = ui.max_rect();
                    max_rect.min.x += size.x / lines_f;
                    max_rect.min.y += size.y / lines_f;
                    max_rect.max.x -= size.x / lines_f;
                    max_rect.max.y -= size.y / lines_f;
                    let sq_size = max_rect.size().min_elem();

                    let incr = sq_size / lines_f;
                    let x_offset = (size.x / 2.0) - ((lines_f + 1.0) / 2.0 * incr);
                    let y_offset = (size.y / 2.0) - ((lines_f + 1.0) / 2.0 * incr);

                    let render = BoardRender {
                        rect: max_rect,
                        lines,
                        lines_f,
                        incr,
                        x_offset,
                        y_offset,
                        sq_size,
                    };

                    render.lines(&painter);

                    if response.clicked() || response.hovered() {
                        if response.clicked() {
                            if let Some(pos) = response.interact_pointer_pos() {
                                let closest = render.closest(&pos);
                                if let Some(point) = closest {
                                    if self
                                        .board
                                        .get_point(point)
                                        .map_or(false, |m| m.color.is_empty())
                                    {
                                        let mut marker = BoardMarker::new(point, Stone::Empty);
                                        if response.ctx.input().modifiers.shift_only() {
                                            *marker.command |=
                                                CommandVariant::MARK | CommandVariant::NOMOVE;
                                            marker.set_oneline_comment("A".to_owned());
                                            self.add_marker(marker);
                                        } else {
                                            marker.color =
                                                Stone::from_bool(self.moves.len() % 2 == 0);
                                            self.add_marker(marker);
                                            if let Some((_variant, index)) =
                                                self.variants.iter().find(|(m, _)| m.point == point)
                                            {
                                                self.graph
                                                    .add_edge(index, &self.graph.current_move())
                                                    .unwrap()
                                            }
                                        }
                                    }
                                    tracing::trace!(%self.board, "added marker");
                                }
                            }
                        }
                    } else if let Some(pos) = response.hover_pos() {
                        if let Some(point) = render.closest(&pos) {
                            if self
                                .board
                                .get_point(point)
                                .map_or(true, |m| m.color.is_empty())
                            {
                                painter.circle(
                                    render.pos_at(&point),
                                    3.0,
                                    Color32::YELLOW,
                                    Stroke::new(2.0, Color32::BLACK),
                                )
                            }
                        }
                    }

                    render.stones(&painter, self);
                    render.marks(&painter, self);
                })
                .response
        });
    }

    /// Add marker
    pub fn add_marker(&mut self, marker: BoardMarker) {
        let existing = if marker.command.is_move() {
            self.graph
                .get_children(&self.graph.current_move())
                .into_iter()
                .find(|f| {
                    if let Some(m) = self.graph.get_move(*f) {
                        m.point == marker.point
                    } else {
                        false
                    }
                })
        } else {
            None
        };
        let idx = if let Some(existing) = existing {
            tracing::debug!(?marker, "marker moved into");
            self.graph.add_move_to_move_list(existing);
            existing
        } else {
            tracing::debug!(?marker, "marker added");
            self.graph.add_move(self.graph.current_move(), marker)
        };
        let (board, moves) = self.graph.as_board(&idx).unwrap();
        self.board = board;
        self.moves = moves;
        self.variants = (0..15 * 15)
            .into_iter()
            .map(|p| Point::from_1d(p, self.board.size()))
            .filter_map(|p| {
                self.graph.get_variant_weird(
                    &self.graph.current_move(),
                    &p,
                    &Stone::from_bool(self.moves.len() % 2 != 0),
                )
            })
            .map(|(b, m)| (b.clone(), m))
            .collect();
    }

    pub fn change_current_move(&mut self, node: &MoveIndex) {
        let mut nodes = self.graph.down_to_root(node);
        nodes.reverse();
        self.graph.set_moves(nodes.len() - 1, nodes);
        let (board, moves) = self.graph.as_board(&self.graph.current_move()).unwrap();
        self.moves = moves;
        self.variants = (0..15 * 15)
            .into_iter()
            .map(|p| Point::from_1d(p, self.board.size()))
            .filter_map(|p| {
                self.graph.get_variant_weird(
                    &self.graph.current_move(),
                    &p,
                    &Stone::from_bool(self.moves.len() % 2 != 0),
                )
            })
            .map(|(b, m)| (b.clone(), m))
            .collect();
        for (marker, _) in &self.variants {
            self.board.set(marker.clone()).unwrap();
        }
        self.board = board;
    }
}

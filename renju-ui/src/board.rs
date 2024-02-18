use egui::{style::Margin, *};
use renju::{
    board::{Board, MoveIndex, Transformation, VariantType},
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
    variants_and_transformations: Vec<(BoardMarker, MoveIndex, Transformation, VariantType)>,
    transform: Transformation,
}

impl UIBoard {
    pub fn new() -> Self {
        Self {
            board: BoardArr::new(15),
            moves: vec![],
            graph: Board::new(),
            variants_and_transformations: vec![],
            transform: Transformation::identity(),
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

    pub fn transform_mut(&mut self) -> &mut Transformation {
        &mut self.transform
    }

    pub fn current_move(&self) -> &BoardMarker {
        self.graph()
            .get_move(self.graph().current_move())
            .expect("oops")
    }
    pub fn current_move_mut(&mut self) -> &mut BoardMarker {
        let curr = self.graph().current_move();
        self.graph_mut().get_move_mut(curr).expect("oops")
    }

    pub fn variants(&self) -> &[(BoardMarker, MoveIndex, Transformation, VariantType)] {
        self.variants_and_transformations.as_ref()
    }

    pub fn transform(&self) -> Transformation {
        self.transform
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
    transform: Transformation,
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
                Rect::from_center_size(self.pos_at(&point).0, Vec2::splat(sq_size * 0.01)),
                Rounding::none(),
                Color32::BLACK,
            );
        }
    }

    /// paints the indexes on the, like 1-15 and A-O
    fn indexes(&self, painter: &Painter) {
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

        let x_range = (x_offset + rect.left());
        let y_range = (y_offset + rect.top());
        for line in 0..lines {
            painter.text(
                Pos2::new(
                    x_range + incr * line as f32,
                    y_range + incr * (lines_f - 1.0) + 20.0,
                ),
                Align2::CENTER_CENTER,
                &char::from(b'A' + line as u8).to_string(),
                FontId::default(),
                Color32::DARK_GRAY,
            );
            painter.text(
                Pos2::new(x_range - 20.0, y_range + incr * line as f32),
                Align2::CENTER_CENTER,
                &(15 - line).to_string(),
                FontId::default(),
                Color32::DARK_GRAY,
            );
        }
    }

    fn marks(&self, painter: &Painter, board: &UIBoard) {
        let BoardRender { lines, sq_size, .. } = *self;

        let children = board.graph.get_children(&board.graph.current_move());
        // find other trees with same outcome if placed
        for (m, mi, transform, variant_type) in &board.variants_and_transformations {
            if children.iter().any(|m| m == mi) {
                continue;
            }
            if board.board.get_point(m.point).unwrap().color.is_empty() && m.command.is_move() {
                let (_, pos) = self.pos_at(&m.point);
                if variant_type == &VariantType::Transformation {
                    //painter.circle(pos, 3.0, Color32::RED, Stroke::new(2.0, Color32::BLACK))
                } else {
                    painter.circle(pos, 3.0, Color32::GOLD, Stroke::new(2.0, Color32::BLACK))
                }
            }
        }
        for child in children {
            let marker = board.graph.get_move(child).unwrap();
            let (_, pos) = self.pos_at(&marker.point);
            painter.circle(pos, 3.0, Color32::WHITE, Stroke::new(2.0, Color32::BLACK))
        }
    }

    /// Returns the position of the center of the point. Not transformed and transformed
    fn pos_at(&self, point: &renju::board_logic::Point) -> (Pos2, Pos2) {
        let BoardRender {
            rect,
            incr,
            x_offset,
            y_offset,
            transform,
            ..
        } = *self;
        (
            Pos2::new(
                x_offset + rect.left() + incr * point.x as f32,
                y_offset + rect.top() + incr * point.y as f32,
            ),
            {
                let point = transform.apply(*point);
                Pos2::new(
                    x_offset + rect.left() + incr * point.x as f32,
                    y_offset + rect.top() + incr * point.y as f32,
                )
            },
        )
    }

    fn center(&self) -> Pos2 {
        let center_x = self.rect.left() + self.x_offset + self.incr * ((self.lines / 2) as f32);
        let center_y = self.rect.top() + self.y_offset + self.incr * ((self.lines / 2) as f32);
        Pos2::new(center_x, center_y)
    }

    /// Get the closest untransformed point to this position.
    fn closest(&self, pos: &Pos2, ui: &Ui) -> Option<Point> {
        let BoardRender {
            rect,
            incr,
            x_offset,
            y_offset,
            lines,
            ..
        } = *self;

        let Pos2 {
            x: center_x,
            y: center_y,
        } = self.center();

        // transform to 0,0 space
        let x = pos.x - center_x;
        let y = pos.y - center_y;

        //let (x, y) = self.transform.apply_f32((x, y));
        // Transform the point to the board pos
        // back to screen space
        let (x, y) = (x + center_x, y + center_y);

        let pos = Pos2::new(x, y);

        let x = x - x_offset - rect.left() + incr / 2.0;
        let y = y - y_offset - rect.top() + incr / 2.0;

        let (x_div, y_div) = (x.div_euclid(incr) as i32, y.div_euclid(incr) as i32);

        tracing::debug!(?x_div, ?y_div, "integer point");
        if let (x_div @ 0.., y_div @ 0..) = (x_div, y_div) {
            if x_div >= lines as i32 || y_div >= lines as i32 {
                return None;
            }
            let point = Point::new(x_div as u32, y_div as u32);
            let (real_pos, trans_pos) = self.pos_at(&point);
            if real_pos.distance(pos) <= (incr / 2.4) {
                Some(self.transform.inverse_apply(point))
            } else {
                None
            }
        } else {
            None
        }
    }

    fn stones(&self, painter: &Painter, board: &UIBoard) {
        for stone in board.board.iter() {
            let (_, pos) = self.pos_at(&stone.point);
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
    #[tracing::instrument(level = "trace", skip(self, ui, just_clicked))]
    pub fn ui(&mut self, ui: &mut egui::Ui, just_clicked: &mut bool) {
        let size = ui.available_size();
        {
            let input = ui.ctx().input();
            match &input.keys_down {
                keys if input.modifiers.shift => match keys {
                    _ if input.key_pressed(Key::ArrowRight) => {
                        let (walked, children) =
                            self.graph.up_to_branch(&self.graph().current_move());
                        if let Some(last) = walked.last() {
                            self.change_current_move(last);
                        }
                    }
                    _ if input.key_pressed(Key::ArrowLeft) => {
                        let down = self.graph().get_down(&self.graph().current_move());
                        if let Some(parent) = &down {
                            self.change_current_move(parent);
                        }
                    }
                    _ => (),
                },
                _ if input.key_pressed(Key::ArrowRight) => {
                    let up = self.graph().get_children(&self.graph().current_move());
                    if let &[child] = &up[..] {
                        self.change_current_move(&child);
                    }
                }
                _ if input.key_pressed(Key::ArrowLeft) => {
                    let down = self.graph().get_parent_strong(&self.graph().current_move());
                    if let Some(parent) = &down {
                        self.change_current_move(parent);
                    }
                }
                _ => (),
            }
        }
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
                        transform: self.transform,
                    };

                    render.lines(&painter);
                    render.indexes(&painter);

                    render.stones(&painter, self);
                    render.marks(&painter, self);

                    if response.clicked() || response.hovered() {
                        if response.clicked() {
                            *just_clicked = true;
                            if let Some(pos) = response.interact_pointer_pos() {
                                let closest = render.closest(&pos, ui);
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
                                            let enter_variant =
                                                if response.ctx.input().modifiers.command {
                                                    if let Some((_, v, t, _vt)) = self
                                                        .variants()
                                                        .iter()
                                                        .find(|(m, _, _t, vt)| m.point == point && vt == &VariantType::Variant)
                                                    {
                                                        Some((*v, t))
                                                    } else {
                                                        None
                                                    }
                                                } else {
                                                    None
                                                };
                                            if let Some((variant, transform)) = enter_variant {
                                                // entering variant, applying transform
                                                tracing::info!("entering variant");
                                                self.transform = self.transform.transform(*transform);
                                                self.change_current_move(&variant);
                                            } else {
                                                marker.color =
                                                    Stone::from_bool(self.moves.len() % 2 == 0);
                                                if let Some((_,mi,t,_)) = self.variants().iter().find(|(m, _, _,vt)|vt == &VariantType::Transformation && m.point == marker.point).cloned() {
                                                    tracing::info!(transform = ?t, "entering transform");
                                                    self.transform = self.transform.transform(t);
                                                    self.change_current_move(&mi);
                                                } else {
                                                    tracing::info!("entering normal move which may be a child");
                                                    let existed = self.add_marker(marker);
                                                    if !existed {
                                                        if let Some((_variant, index, _transform, variant_type)) = self
                                                        .variants_and_transformations
                                                        .iter()
                                                        .find(|(m, _, _transform, variant_type)| m.point == point)
                                                        {
                                                            if let Err(e) = self.graph.add_edge(
                                                                index,
                                                                &self.graph.current_move(),
                                                            ) {
                                                                tracing::error!(error = ?e, "Oh no!")
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            }
                                        }
                                    tracing::trace!(%self.board, "added marker");
                                }
                            }
                        } else if let Some(pos) = response.hover_pos() {
                            if let Some(closest) = render.closest(&pos, ui) {
                                if self
                                    .board
                                    .get_point(closest)
                                    .map_or(true, |m| m.color.is_empty())
                                {
                                    painter.circle(
                                        render.pos_at(&closest).1,
                                        3.0,
                                        Color32::BLUE,
                                        Stroke::new(2.0, Color32::BLACK),
                                    )
                                }
                                if self.variants_and_transformations.iter().any(|(m, _, _, variant_type)| m.point == closest && variant_type == &VariantType::Variant) {
                                    egui::containers::show_tooltip_at_pointer(
                                        ui.ctx(),
                                        ui.id().with("__tooltip"),
                                        |ui| Label::new("enter branch with ⌘ + click").ui(ui),
                                    );
                                }
                            }
                        }
                    }

                })
                .response
        });
    }

    /// Add marker, returns true if the marker already existed in the graph
    #[tracing::instrument(skip(self))]
    pub fn add_marker(&mut self, marker: BoardMarker) -> bool {
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
        self.update_variants();
        existing.is_some()
    }

    #[tracing::instrument(skip(self))]
    pub fn change_current_move(&mut self, node: &MoveIndex) {
        let mut nodes = self.graph.down_to_root(node);
        nodes.reverse();
        self.graph.set_moves(nodes.len() - 1, nodes);
        let (board, moves) = self.graph.as_board(&self.graph.current_move()).unwrap();
        self.moves = moves;
        self.board = board;
        self.update_variants();
    }

    #[tracing::instrument(skip(self))]
    pub fn update_variants(&mut self) {
        let current_move = self.graph.current_move();
        self.variants_and_transformations = self
            .graph
            .get_variants_and_transformations(current_move)
            .unwrap();
        let children = self
            .graph
            .get_children(&current_move)
            .into_iter()
            .filter_map(|mi| self.graph.get_move(mi));
        self.variants_and_transformations
            .retain(|(p, .., typ)| !children.clone().any(|other| p.point == other.point && typ == &VariantType::Transformation));

        // for (marker, _, variant, _) in &self.variants_and_transformations {
        //     if let Some(marker) = self.board.get_point(marker.point) {
        //         if !(marker.command.is_move()
        //             || marker.command.is_mark()
        //             || !marker.color.is_empty())
        //         {
        //             self.board.set(marker.clone()).unwrap();
        //         }
        //     }
        // }
    }
}

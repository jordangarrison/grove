#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct TextSelectionPoint {
    pub(super) line: usize,
    pub(super) col: usize,
}

impl TextSelectionPoint {
    fn before(self, other: Self) -> bool {
        self.line < other.line || (self.line == other.line && self.col < other.col)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(super) struct TextSelectionState {
    pub(super) active: bool,
    pub(super) start: Option<TextSelectionPoint>,
    pub(super) end: Option<TextSelectionPoint>,
    pub(super) anchor: Option<TextSelectionPoint>,
}

impl TextSelectionState {
    pub(super) fn clear(&mut self) {
        self.active = false;
        self.start = None;
        self.end = None;
        self.anchor = None;
    }

    pub(super) fn has_selection(&self) -> bool {
        self.start.is_some() && self.end.is_some()
    }

    pub(super) fn prepare_drag(&mut self, point: TextSelectionPoint) {
        self.active = false;
        self.start = None;
        self.end = None;
        self.anchor = Some(point);
    }

    pub(super) fn handle_drag(&mut self, point: TextSelectionPoint) {
        let Some(anchor) = self.anchor else {
            return;
        };
        if self.start.is_none() {
            self.start = Some(anchor);
            self.end = Some(anchor);
        }

        self.active = true;
        if point.before(anchor) {
            self.start = Some(point);
            self.end = Some(anchor);
        } else {
            self.start = Some(anchor);
            self.end = Some(point);
        }
    }

    pub(super) fn finish_drag(&mut self) {
        if self.start.is_none() {
            self.clear();
            return;
        }

        self.active = false;
        self.anchor = None;
    }

    pub(super) fn bounds(&self) -> Option<(TextSelectionPoint, TextSelectionPoint)> {
        Some((self.start?, self.end?))
    }

    pub(super) fn line_selection_cols(&self, line_idx: usize) -> Option<(usize, Option<usize>)> {
        let (start, end) = self.bounds()?;
        if line_idx < start.line || line_idx > end.line {
            return None;
        }

        if start.line == end.line {
            return Some((start.col, Some(end.col)));
        }
        if line_idx == start.line {
            return Some((start.col, None));
        }
        if line_idx == end.line {
            return Some((0, Some(end.col)));
        }

        Some((0, None))
    }
}

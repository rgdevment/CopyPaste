use crate::model::Rows;
use slint::{Model, ModelTracker};
use std::rc::Rc;

pub const ACROSS: usize = 2;

const _: () = assert!(ACROSS >= 2);

pub struct Paired {
    rows: Rc<Rows>,
}

impl Paired {
    pub fn over(rows: Rc<Rows>) -> Rc<Self> {
        Rc::new(Self { rows })
    }

    pub fn lines(&self) -> usize {
        lines_for(self.rows.row_count())
    }
}

pub fn lines_for(cells: usize) -> usize {
    cells.div_ceil(ACROSS)
}

impl Model for Paired {
    type Data = i32;

    fn row_count(&self) -> usize {
        self.lines()
    }

    fn row_data(&self, index: usize) -> Option<i32> {
        if index >= self.lines() {
            return None;
        }
        let _ = self.rows.row_data(index * ACROSS);
        i32::try_from(index).ok()
    }

    fn model_tracker(&self) -> &dyn ModelTracker {
        self.rows.model_tracker()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
#[path = "paired_test.rs"]
mod tests;

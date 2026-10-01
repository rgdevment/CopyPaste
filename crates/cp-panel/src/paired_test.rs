use super::*;
use crate::model::Metrics;
use cp_store::{Filter, Store};

const SIZES: Metrics = Metrics {
    tall: 146.0,
    json: 112.0,
    plain: 86.0,
    found: 68.0,
    frame: 50.0,
    line: 18.0,
};

fn grid_over(count: usize) -> (Rc<Rows>, Rc<Paired>) {
    let store = Store::in_memory().expect("esquema");
    for at in 0..count {
        store
            .insert_text(&format!("u{at}"), &format!("imagen {at}"), at as i64)
            .expect("insert");
    }
    let rows = Rows::open(Rc::new(store), Filter::default(), 1_000, SIZES);
    let grid = Paired::over(rows.clone());
    (rows, grid)
}

#[test]
fn a_grid_of_two_across_needs_half_the_lines_rounded_up() {
    assert_eq!(lines_for(0), 0);
    assert_eq!(lines_for(1), 1, "one cell still takes a whole line");
    assert_eq!(lines_for(2), 1);
    assert_eq!(lines_for(3), 2, "the odd one gets a line of its own");
    assert_eq!(lines_for(4), 2);
    assert_eq!(lines_for(9), 5);
}

#[test]
fn the_count_never_loses_a_cell() {
    for cells in 0..40usize {
        let lines = lines_for(cells);
        assert!(
            lines * ACROSS >= cells,
            "{cells} cells would not fit in {lines} lines"
        );
        if cells > 0 {
            assert!(
                (lines - 1) * ACROSS < cells,
                "{lines} lines for {cells} cells leaves one empty"
            );
        }
    }
}

#[test]
fn the_grid_has_a_line_for_every_pair_of_rows() {
    let (rows, grid) = grid_over(5);
    assert_eq!(rows.row_count(), 5);
    assert_eq!(grid.row_count(), 3, "two pairs and a lonely one");
    assert_eq!(grid.lines(), grid.row_count());
}

#[test]
fn a_line_answers_with_its_own_number_so_its_cells_can_find_themselves() {
    let (_, grid) = grid_over(5);
    assert_eq!(grid.row_data(0), Some(0));
    assert_eq!(grid.row_data(1), Some(1));
    assert_eq!(grid.row_data(2), Some(2));
}

#[test]
fn there_is_no_line_past_the_last_pair() {
    let (_, grid) = grid_over(5);
    assert_eq!(grid.row_data(3), None, "three lines hold the five cells");
    assert_eq!(grid.row_data(900), None);
}

#[test]
fn an_empty_history_draws_no_lines_at_all() {
    let (_, grid) = grid_over(0);
    assert_eq!(grid.row_count(), 0);
    assert_eq!(grid.row_data(0), None);
}

#[test]
fn the_grid_counts_what_the_list_holds_now() {
    let (rows, grid) = grid_over(300);
    assert_eq!(grid.row_count(), lines_for(rows.row_count()));
    assert!(
        rows.row_count() < 300,
        "the list still has pages to come, so the count cannot be a snapshot"
    );
}

#[test]
fn the_view_can_recognise_the_grid_it_was_given() {
    let (_, grid) = grid_over(2);
    let any: Rc<dyn Model<Data = i32>> = grid.clone();
    assert!(any.as_any().downcast_ref::<Paired>().is_some());
}

#[test]
fn the_grid_listens_to_the_same_tracker_as_the_list() {
    let (rows, grid) = grid_over(4);
    let one = std::ptr::from_ref(grid.model_tracker()).cast::<()>();
    let other = std::ptr::from_ref(rows.model_tracker()).cast::<()>();
    assert!(
        std::ptr::eq(one, other),
        "a grid with its own tracker would never hear a new page"
    );
}

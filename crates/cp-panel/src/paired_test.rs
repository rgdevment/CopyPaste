use super::*;

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

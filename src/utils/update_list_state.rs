use crate::events::event::Direction;
use ratatui::widgets::ListState;

pub fn update_list(list_state: &mut ListState, direction: Direction, len: usize) {
    match direction {
        Direction::Up => list_state.select_previous(),
        Direction::Down => select_next(list_state, len),
    }
}

pub fn update_noneable_list(list_state: &mut ListState, direction: Direction, len: usize) {
    let selected = list_state.selected();
    if len == 0 {
        list_state.select(None);
        return;
    }

    match direction {
        Direction::Up => {
            if selected.unwrap_or(0) > 0 {
                list_state.select_previous();
            } else {
                list_state.select(None);
            }
        }
        Direction::Down => select_next(list_state, len),
    }
}

fn select_next(list_state: &mut ListState, len: usize) {
    let selected = list_state.selected().unwrap_or(0);
    let max_selection = len.saturating_sub(1);
    if selected == max_selection {
        list_state.select(Some(max_selection));
    } else {
        list_state.select_next();
    }
}

#[cfg(test)]
mod tests {
    use crate::events::event::Direction;
    use crate::utils::update_list_state::{select_next, update_list, update_noneable_list};
    use ratatui::widgets::ListState;
    use test_case::test_case;

    #[test_case(None, 2, Some(0))]
    #[test_case(Some(1), 2, Some(1))]
    fn update_list_selects_as_expected(
        initial_selected: Option<usize>,
        len: usize,
        expected_selected: Option<usize>,
    ) {
        let mut list = ListState::default();
        list.select(initial_selected);
        update_list(&mut list, Direction::Down, len);
        assert_eq!(list.selected(), expected_selected);
    }

    #[test_case(None, 1, Direction::Up, None)]
    #[test_case(Some(1), 2, Direction::Up, Some(0))]
    #[test_case(Some(0), 2, Direction::Up, None)]
    #[test_case(None, 1, Direction::Down, Some(0))]
    #[test_case(Some(0), 2, Direction::Down, Some(1))]
    #[test_case(Some(1), 2, Direction::Down, Some(1))]
    #[test_case(None, 0, Direction::Down, None)]
    fn update_noneable_list_selects_as_expected(
        initial_selected: Option<usize>,
        len: usize,
        dir: Direction,
        expected_selected: Option<usize>,
    ) {
        let mut list = ListState::default();
        list.select(initial_selected);
        update_noneable_list(&mut list, dir, len);
        assert_eq!(list.selected(), expected_selected);
    }

    #[test_case(None, 2, Some(0))]
    #[test_case(Some(0), 2, Some(1))]
    #[test_case(Some(1), 2, Some(1))]
    fn selects_next_selects_as_expected(
        initial_selected: Option<usize>,
        len: usize,
        expected_selected: Option<usize>,
    ) {
        let mut list = ListState::default();
        list.select(initial_selected);
        select_next(&mut list, len);
        assert_eq!(list.selected(), expected_selected);
    }
}

use ratatui::widgets::ListState;
use crate::events::event::Direction;

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
    let n = len.saturating_sub(1);
    if selected == n {
        list_state.select(Some(n));
    } else {
        list_state.select_next();
    }
}
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};
use crate::App;

pub mod device_list;
pub mod main_terminal;

use device_list::*;
use main_terminal::*;

pub fn center_ui<B: Backend>(f: &mut Frame<B>, size: Rect, app: &mut App) {
    let chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .margin(0)
                        .constraints(
                            [
                                Constraint::Percentage(40),
                                Constraint::Percentage(60)
                            ].as_ref()
                        )
                        .split(size);
    
    devices_ui(f, chunks[0], app);
    main_terminal_ui(f, chunks[1], app);
}
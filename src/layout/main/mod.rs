pub mod indicator;
pub mod center_widget;
pub mod flash_terminal;
pub mod help;

use std::time::SystemTime;

use crossterm::event::{KeyEvent, KeyCode};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders},
    Frame,
};
use crate::{
    App,
    ui_selection::*, module_detect::refresh_devlist, jetson::Signal,
};
use indicator::*;
use center_widget::*;
use flash_terminal::*;
use help::*;

pub fn main_ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let frame_size = &f.size();
    let title_height = 1;
    let indicator_height = 1;
    let help_height = 2;
    let main_height = frame_size.height - title_height - indicator_height - help_height;
    let center_height = main_height * 5 / 10;
    let terminal_height = main_height - center_height;

    let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints(
                        [
                            Constraint::Length(title_height),
                            Constraint::Length(indicator_height),
                            Constraint::Length(center_height),
                            Constraint::Length(terminal_height),
                            Constraint::Length(help_height),
                        ].as_ref()
                    )
                    .split(f.size());
    
    let title = Block::default()
                    .title("Supergate Jetson Devkit Flashing Tool")
                    .borders(Borders::NONE);


    f.render_widget(title, chunks[0]);
    indicator_ui(f, chunks[1], app);
    center_ui(f, chunks[2], app);
    terminal_ui(f, chunks[3], app);
    help_ui(f, chunks[4], app);

}

pub fn control(app: &mut App, key: KeyEvent) -> Option<()> {
    match app.selection.focused {
        UISelection::DeviceList(None) => {
            match key.code {
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                    app.index = 2;
                },
                KeyCode::F(5) | KeyCode::Char('r') | KeyCode::Char('R') => {
                    // refresh
                    let now = SystemTime::now();
                    refresh_devlist(app);
                    let elapsed_time = now.elapsed().unwrap().as_millis().to_string();
                    app.tx.send(Signal::Message(String::from("Refreshing device list takes ") + &elapsed_time + " milliseconds.\n")).unwrap();
                }
                _ => {
                    center_widget::device_list::control(app, key);
                }
            }
        }
        UISelection::DeviceList(Some(_)) => {
            center_widget::device_list::control(app, key);
        },
        UISelection::FlashTerminal => {
            center_widget::main_terminal::control(app, key);
        },
        _ => {}
    }
    None
}

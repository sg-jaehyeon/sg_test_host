use std::time::SystemTime;

use crossterm::event::{KeyCode, KeyEvent};
use tui::{
    backend::Backend,
    layout::{Rect, Margin},
    widgets::{Block, Borders, List},
    Frame
};
use crate::{App, UISelectionModel, UISelection, test::env_setup::{check_env, setup_workspace}, app::InstallStatus, jetson::FlashStatus};
use crate::jetson::Signal;
use crate::module_detect::refresh_devlist;

pub fn devices_ui<B: Backend>(f: &mut Frame<B>, size: Rect, app: &mut App) {
    let block = Block::default()
                        .title("Devices")
                        .borders(Borders::ALL);

    devices_inner_ui(f, block.inner(size), app);
    f.render_widget(block, size);
}

fn devices_inner_ui<B: Backend>(f: &mut Frame<B>, size: Rect, app: &mut App) {
    let inner_size = size.inner(&Margin {
        vertical: 0,
        horizontal: 1,
    });
    match app.devlist.len() {
        0 => {
            let notice = Block::default()
                                .title("No devices detected");

            f.render_widget(notice, inner_size);
        },
        _ => {
            let list = List::new(app.list());
            f.render_widget(list, inner_size);
        }
    }
}

pub fn control(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Up => {
            let index = app.previous_device();
            if index.is_some() {
                app.select(UISelectionModel { focused: UISelection::DeviceList(None), current: UISelection::DeviceList(Some(index.unwrap())) });
            }
        },
        KeyCode::Down => {
            let index = app.next_device();
            if index.is_some() {
                app.select(UISelectionModel { focused: UISelection::DeviceList(None), current: UISelection::DeviceList(Some(index.unwrap())) });
            }
        },
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
            // Quit
            app.index = 2;
        },
        KeyCode::Enter => {
            if let InstallStatus::Installing(_) = app.install_status {
                app.tx.send(Signal::Message(String::from("Please wait for environment setup finished.\n"))).unwrap();
            } else {
                match app.flash_status {
                    FlashStatus::Wait => {
                        if let UISelection::DeviceList(_) = app.selection.current {
                            app.index = 1;
                        }
                    },
                    FlashStatus::Flashing => {
                        app.tx.send(Signal::Message(String::from("Flashing in progress.\n"))).unwrap();
                    },
                    FlashStatus::Finished => {
                        app.tx.send(Signal::Message(String::from("Please restart program.\n"))).unwrap();
                    }
                    _ => {}
                }
            }
        },
        KeyCode::F(5) => {
            refresh_devlist(app);
        },
        KeyCode::F(6) => {
            if app.installer.is_none() && !check_env(app.tx.clone()) && app.install_status == InstallStatus::NotInstalled {
                let tx = app.create_new_publisher();
                tx.send(Signal::EnvironmentInstalling(SystemTime::now())).unwrap();
                app.installer = Some(setup_workspace(tx, "."));
            }
        }
        _ => {},
    }
}
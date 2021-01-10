#![windows_subsystem = "windows"]
use native_windows_gui as nwg;

use nwg::NativeUi;
use reqwest::blocking::Client;
use std::{
    fs::File,
    io::{prelude::*, BufReader},
    path::{Path, PathBuf},
};
use sysinfo::{ProcessExt, System, SystemExt};

#[derive(Default)]
pub struct SystemTray {
    window: nwg::MessageWindow,
    icon: nwg::Icon,
    tray: nwg::TrayNotification,
    tray_menu: nwg::Menu,
    tray_item_activate_skin_boost: nwg::MenuItem,
    tray_item_exit_application: nwg::MenuItem,
}

impl SystemTray {
    fn show_menu(&self) {
        let (x, y) = nwg::GlobalCursor::position();
        self.tray_menu.popup(x, y);
    }

    fn activate_skin_boost(&self) {
        let mut sys: System = System::new_all();
        sys.refresh_all();
        for (_, process) in sys.get_processes() {
            let process_executable: &str = process.name();
            if process_executable == "LeagueClient.exe" {
                let process_path: &Path = process.root();
                let mut path: PathBuf = PathBuf::new();
                path.push(process_path);
                path.push("lockfile");

                if path.exists() {
                    let file = File::open(path).expect("Lockfile not found!");
                    let buf = BufReader::new(file);

                    let lockfile: Vec<String> = buf
                        .lines()
                        .map(|l| l.expect("Could not parse lockfile!"))
                        .collect();
                    let lockfile_data: Vec<&str> = lockfile[0].split(':').collect();
                    let lcu_port: String = lockfile_data[2].to_string();
                    let lcu_password: String = lockfile_data[3].to_string();
                    let lcu_protocol: String = lockfile_data[4].to_string();

                    let request_url: String = format!(
                        "{}://127.0.0.1:{}/lol-champ-select/v1/team-boost/purchase",
                        lcu_protocol, lcu_port
                    );
                    Client::builder()
                        .danger_accept_invalid_certs(true)
                        .build()
                        .unwrap()
                        .post(&request_url)
                        .basic_auth("riot", Some(lcu_password))
                        .send()
                        .unwrap();
                }
            }
        }
    }

    fn exit_application(&self) {
        nwg::stop_thread_dispatch();
    }
}

mod system_tray_ui {
    use super::*;
    use native_windows_gui as nwg;
    use std::cell::RefCell;
    use std::ops::Deref;
    use std::rc::Rc;

    pub struct SystemTrayUi {
        inner: Rc<SystemTray>,
        default_handler: RefCell<Vec<nwg::EventHandler>>,
    }

    impl nwg::NativeUi<SystemTrayUi> for SystemTray {
        fn build_ui(mut data: SystemTray) -> Result<SystemTrayUi, nwg::NwgError> {
            use nwg::Event as E;

            nwg::Icon::builder()
                .source_bin(Some(include_bytes!("../assets/icon.ico")))
                .build(&mut data.icon)?;
            nwg::MessageWindow::builder().build(&mut data.window)?;

            nwg::TrayNotification::builder()
                .parent(&data.window)
                .icon(Some(&data.icon))
                .tip(Some("Skin Booster"))
                .build(&mut data.tray)?;

            nwg::Menu::builder()
                .popup(true)
                .parent(&data.window)
                .build(&mut data.tray_menu)?;

            nwg::MenuItem::builder()
                .text("Activate Skin Boost")
                .parent(&data.tray_menu)
                .build(&mut data.tray_item_activate_skin_boost)?;

            nwg::MenuItem::builder()
                .text("Exit")
                .parent(&data.tray_menu)
                .build(&mut data.tray_item_exit_application)?;

            let ui = SystemTrayUi {
                inner: Rc::new(data),
                default_handler: Default::default(),
            };

            let evt_ui = Rc::downgrade(&ui.inner);
            let handle_events = move |evt, _evt_data, handle| {
                if let Some(evt_ui) = evt_ui.upgrade() {
                    match evt {
                        E::OnContextMenu => {
                            if &handle == &evt_ui.tray {
                                SystemTray::show_menu(&evt_ui);
                            }
                        }
                        E::OnMenuItemSelected => {
                            if &handle == &evt_ui.tray_item_activate_skin_boost {
                                SystemTray::activate_skin_boost(&evt_ui);
                            }
                            if &handle == &evt_ui.tray_item_exit_application {
                                SystemTray::exit_application(&evt_ui);
                            }
                        }
                        _ => {}
                    }
                }
            };

            ui.default_handler
                .borrow_mut()
                .push(nwg::full_bind_event_handler(
                    &ui.window.handle,
                    handle_events,
                ));

            return Ok(ui);
        }
    }

    impl Drop for SystemTrayUi {
        fn drop(&mut self) {
            let mut handlers = self.default_handler.borrow_mut();
            for handler in handlers.drain(0..) {
                nwg::unbind_event_handler(&handler);
            }
        }
    }

    impl Deref for SystemTrayUi {
        type Target = SystemTray;

        fn deref(&self) -> &SystemTray {
            &self.inner
        }
    }
}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    let _ui = SystemTray::build_ui(Default::default()).expect("Failed to build UI");
    nwg::dispatch_thread_events();
}

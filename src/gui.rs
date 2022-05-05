// Copyright (c) 2022 aiocat
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use crate::injector;
use crate::msgbox;
use crate::spoof;

use eframe::{egui, egui::containers::ScrollArea, epaint, IconData};
use rfd::FileDialog;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::Path;
use sysinfo::{PidExt, ProcessExt, System, SystemExt};

#[derive(Debug, std::cmp::PartialEq)]
enum InjectionMethods {
    CreateRemoteThread,
    RtlCreateUserThread,
    QueueUserAPC,
    NtCreateThreadEx,
}

#[derive(Debug, std::cmp::PartialEq)]
enum SelectGui {
    Injection,
    Processes,
}

// this struct holds application data for window lifecycle
pub struct DLLCrabWindow {
    pid: String,
    dll_name: String,
    dll_path: String,
    process_filter: String,
    system: System,
    processes: HashMap<u32, String>,
    close_after_injection: bool,
    spoofing: bool,
    selected_method: InjectionMethods,
    selected_gui: SelectGui,
}

// this function runs a new egui instance
pub fn draw_window() {
    // window options
    let options = eframe::NativeOptions {
        resizable: true,
        initial_window_size: Some(egui::Vec2 { x: 300.0, y: 300.0 }),
        min_window_size: Some(egui::Vec2 { x: 300.0, y: 300.0 }),
        icon_data: load_icon(".\\dll-crab.ico"),
        ..Default::default()
    };

    // draw window
    eframe::run_native(
        "DLL Crab",
        options,
        Box::new(|ctx: &eframe::CreationContext| {
            let mut style = egui::Style::default();
            style.visuals.dark_mode = true;
            ctx.egui_ctx.set_style(style);

            Box::new(DLLCrabWindow::default())
        }),
    );
}

impl Default for DLLCrabWindow {
    fn default() -> Self {
        let mut data = Self {
            pid: String::from("0"),
            dll_name: String::from("None"),
            dll_path: String::new(),
            process_filter: String::from("Filter"),
            system: System::new_all(),
            processes: HashMap::new(),
            close_after_injection: false,
            spoofing: false,
            selected_method: InjectionMethods::CreateRemoteThread,
            selected_gui: SelectGui::Injection,
        };

        data.system.refresh_all();
        for (pid, process) in data.system.processes() {
            data.processes
                .insert(pid.as_u32(), process.name().to_string());
        }

        data
    }
}

// injection function
impl DLLCrabWindow {
    pub fn inject(&self) {
        // check if ends with dll
        if Path::new(&self.dll_path)
            .extension()
            .unwrap_or_else(|| OsStr::new(""))
            != "dll"
        {
            unsafe {
                msgbox::error("Library path is invalid. Please select a library to continue...");
            };
            return;
        }

        // check pid format
        let pid = self.pid.parse::<u32>();
        if pid.is_err() {
            unsafe {
                msgbox::error("PID format is invalid. Please check your input!");
            };
            return;
        }

        // run injector
        let pid: u32 = pid.unwrap();
        let function_to_use = match self.selected_method {
            InjectionMethods::CreateRemoteThread => injector::inject_create_remote_thread,
            InjectionMethods::RtlCreateUserThread => injector::inject_rtl_create_user_thread,
            InjectionMethods::QueueUserAPC => injector::inject_queue_user_apc,
            InjectionMethods::NtCreateThreadEx => injector::inject_nt_create_thread_ex,
        };

        // get dll path
        let dll_path = if self.spoofing {
            spoof::spoof_dll(self.dll_path.clone())
        } else {
            self.dll_path.clone()
        };

        let result = function_to_use(pid, &dll_path);

        // check result
        unsafe {
            if !result {
                msgbox::error("Injection failed. Maybe PID is invalid?");
            } else {
                msgbox::info("Library is injected to the process.");
            }
        }
    }
}

// import eframe's lifecycle
impl eframe::App for DLLCrabWindow {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let main_frame = egui::containers::Frame {
            rounding: egui::Rounding::none(),
            shadow: epaint::Shadow {
                extrusion: 0.0,
                color: egui::Color32::BLACK,
            },
            ..egui::containers::Frame::window(&egui::Style::default())
        };

        // bottom panel
        egui::TopBottomPanel::bottom("bottom")
            .frame(main_frame)
            .show(ctx, |ui: &mut egui::Ui| {
                ui.small("v1.3.2");
                egui::menu::bar(ui, |ui: &mut egui::Ui| {
                    ui.hyperlink_to("Source Code", "https://github.com/aiocat/dll-crab");
                    ui.hyperlink_to(
                        "Credits",
                        "https://github.com/aiocat/dll-crab/graphs/contributors",
                    );
                    ui.hyperlink_to(
                        "License",
                        "https://github.com/aiocat/dll-crab/blob/main/LICENSE",
                    );
                });
            });

        // main part
        egui::CentralPanel::default()
            .frame(main_frame)
            .show(ctx, |ui: &mut egui::Ui| {
                self.main_ui(ui, frame);
            });
    }
}

impl DLLCrabWindow {
    fn main_ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.selected_gui, SelectGui::Injection, "Injection");
            ui.selectable_value(&mut self.selected_gui, SelectGui::Processes, "Processes");
        });
        ui.separator();

        match self.selected_gui {
            SelectGui::Injection => self.injection_ui(ui, frame),
            SelectGui::Processes => self.processes_ui(ui),
        }
    }

    fn injection_ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        // title
        ui.heading("Injection");
        ui.add_space(4.0);

        // dll name label
        ui.horizontal(|ui: &mut egui::Ui| {
            ui.label("Selected DLL: ");
            ui.label(&self.dll_name);
        });

        // application pid textbox
        ui.horizontal(|ui: &mut egui::Ui| {
            ui.label("Application PID: ");
            ui.text_edit_singleline(&mut self.pid);
        });

        // injection method combobox
        ui.horizontal(|ui: &mut egui::Ui| {
            ui.label("Injection Method: ");

            // combobox
            egui::ComboBox::from_label("")
                .selected_text(format!("{:?}", self.selected_method))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.selected_method,
                        InjectionMethods::CreateRemoteThread,
                        "CreateRemoteThread",
                    );
                    ui.selectable_value(
                        &mut self.selected_method,
                        InjectionMethods::RtlCreateUserThread,
                        "RtlCreateUserThread",
                    );
                    ui.selectable_value(
                        &mut self.selected_method,
                        InjectionMethods::QueueUserAPC,
                        "QueueUserAPC",
                    );
                    ui.selectable_value(
                        &mut self.selected_method,
                        InjectionMethods::NtCreateThreadEx,
                        "NtCreateThreadEx",
                    );
                });
        });

        // checkboxes
        ui.horizontal(|ui: &mut egui::Ui| {
            // dll spoof
            ui.checkbox(&mut self.spoofing, "Spoof DLL");

            // set close_after_injection
            ui.checkbox(&mut self.close_after_injection, "Close After Injection");
        });

        // display buttons as inline-block
        ui.horizontal(|ui: &mut egui::Ui| {
            // open dll file dialog
            if ui.button("Open DLL").clicked() {
                if let Some(path) = FileDialog::new()
                    .add_filter("Dynamic Library", &["dll"])
                    .pick_file()
                {
                    self.dll_name = path.file_name().unwrap().to_str().unwrap().to_owned();
                    self.dll_path = path.display().to_string();
                }
            }

            // inject dll
            if ui.button("Inject").clicked() {
                self.inject();

                if self.close_after_injection {
                    frame.quit();
                }
            }
        });
    }

    fn processes_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Processes");
        ui.add_space(4.0);
        ui.horizontal(|ui: &mut egui::Ui| {
            // refresh list button
            if ui.button("Refresh").clicked() {
                self.system.refresh_all();
                self.process_filter = String::from("Filter");
                self.processes = HashMap::new();
                for (pid, process) in self.system.processes() {
                    self.processes
                        .insert(pid.as_u32(), process.name().to_string());
                }
            }

            // filter list
            if ui.button("Filter").clicked() {
                self.system.refresh_all();

                self.processes = HashMap::new();
                for (pid, process) in self.system.processes() {
                    if process
                        .name()
                        .to_lowercase()
                        .contains(&self.process_filter.to_lowercase())
                    {
                        self.processes
                            .insert(pid.as_u32(), process.name().to_string());
                    }
                }
            }

            // filter list by process name textbox
            ui.text_edit_singleline(&mut self.process_filter);
        });

        // process list
        ui.add_space(4.0);
        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show_viewport(ui, |ui: &mut eframe::egui::Ui, _| {
                let font_id = egui::TextStyle::Body.resolve(ui.style());
                let row_height = ui.fonts().row_height(&font_id) + ui.spacing().item_spacing.y;

                ui.set_height(self.processes.len() as f32 * (row_height * 1.5));

                for (pid, process) in &self.processes {
                    ui.horizontal(|ui| {
                        ui.label(pid.to_string());

                        // load pid
                        if ui.link(process).clicked() {
                            self.pid = pid.to_string();
                            self.selected_gui = SelectGui::Injection;
                        }
                    });
                }
            });
    }
}

fn load_icon(path: &str) -> Option<IconData> {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open(path)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    Some(IconData {
        rgba: icon_rgba,
        width: icon_width,
        height: icon_height,
    })
}

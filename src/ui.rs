use eframe::{egui, egui::{TextEdit, RichText, Color32}};
use serde_json::Value;
use std::time::Instant;

use crate::data::{self, AuthConfig, AuthType, Method, RequestProfile};
use crate::comm::{self, UserAction, UiMessage};

#[derive(Clone, Copy, PartialEq)]
enum BodyMode { Json, Form }

#[derive(Clone, Default)]
struct FormField { enabled: bool, name: String, value: String }

pub struct AppState {
    // Request
    url: String,
    method: Method,
    headers_text: String,
    body_text: String,
    body_mode: BodyMode,
    form_fields: Vec<FormField>,
    auth: AuthConfig,

    // Response
    prev_response: Option<Value>,
    response: Option<Value>,
    raw_response: String,
    status_line: String,
    elapsed_ms: Option<u128>,

    // Diff cache
    text_diff: Option<String>,
    json_patch: Option<String>,
    // Inline diff mapping (JSON Pointer path -> op)
    changed_ops: std::collections::HashMap<String, String>,
    // Previous values at changed paths (for inline old-value display)
    prev_values: std::collections::HashMap<String, Value>,

    // Async comms
    tx: std::sync::mpsc::Sender<UserAction>,
    rx_ui: std::sync::mpsc::Receiver<UiMessage>,

    // UI state
    resp_tab: usize,
    compact_ui: bool,
    diff_inline: bool,
    filter_tree: bool,
    // Window
    always_on_top: bool,

    // Extensions state
    profiles: Vec<RequestProfile>,
    profile_name: String,
    // Project/Profile management (folder-based)
    current_project: String,
    available_projects: Vec<String>,
    available_profiles: Vec<String>,
    selected_profile: String,
    // Profiles explorer UI state
    new_project_name: String,
    new_profile_name: String,
    rename_project_name: String,
    rename_profile_name: String,

    search_text: String,
    search_results: Vec<String>,
    jsonpath_query: String,
    jmespath_query: String,
    jsonpath_result: String,
    jmespath_result: String,
    auto_poll: bool,
    poll_interval_sec: f32,
    last_poll_instant: Option<Instant>,

    // cURL import
    curl_input: String,
}

impl Default for AppState {
    fn default() -> Self {
        let (tx, rx_ui) = comm::spawn_comm();

        let mut app = Self {
            url: "https://httpbin.org/get".into(),
            method: Method::GET,
            headers_text: "User-Agent: api-tester\nAccept: application/json".into(),
            body_text: "{\n  \"hello\": \"world\"\n}".into(),
            body_mode: BodyMode::Json,
            form_fields: vec![FormField { enabled: true, name: String::new(), value: String::new() }],
            auth: AuthConfig::default(),
            prev_response: None,
            response: None,
            raw_response: String::new(),
            status_line: String::new(),
            elapsed_ms: None,
            text_diff: None,
            json_patch: None,
            changed_ops: std::collections::HashMap::new(),
            prev_values: std::collections::HashMap::new(),
            tx,
            rx_ui,
            resp_tab: 0,
            compact_ui: true,
            diff_inline: false,
            filter_tree: false,
            always_on_top: false,
            profiles: vec![],
            profile_name: String::new(),
            current_project: "default".to_string(),
            available_projects: vec![],
            available_profiles: vec![],
            selected_profile: String::new(),
            new_project_name: String::new(),
            new_profile_name: String::new(),
            rename_project_name: String::new(),
            rename_profile_name: String::new(),
            search_text: String::new(),
            search_results: vec![],
            jsonpath_query: String::new(),
            jmespath_query: String::new(),
            jsonpath_result: String::new(),
            jmespath_result: String::new(),
            auto_poll: false,
            poll_interval_sec: 5.0,
            last_poll_instant: None,
            curl_input: String::new(),
        };
        app.refresh_projects();
        // Initialize form fields from default body_text for consistency
        app.update_form_from_body();
        app
    }
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if self.compact_ui {
            let mut style = (*ctx.style()).clone();
            style.spacing.item_spacing = egui::vec2(4.0, 4.0);
            style.spacing.indent = 8.0;
            style.spacing.button_padding = egui::vec2(6.0, 3.0);
            style.text_styles.iter_mut().for_each(|(_, v)| v.size = (v.size * 0.9).max(10.0));
            ctx.set_style(style);
        }

        // Auto polling
        if self.auto_poll {
            let should_poll = match self.last_poll_instant {
                Some(t) => t.elapsed().as_secs_f32() >= self.poll_interval_sec,
                None => true,
            };
            if should_poll {
                self.send_current_request();
                self.last_poll_instant = Some(Instant::now());
            }
        }

        // Receive responses
        while let Ok(msg) = self.rx_ui.try_recv() {
            if let UiMessage::Response { status_line, raw, parsed, elapsed_ms } = msg {
                self.status_line = status_line;
                self.elapsed_ms = Some(elapsed_ms);

                self.prev_response = self.response.take();
                self.response = parsed;
                self.raw_response = raw;

                self.update_diffs();
                self.update_search_results();
            }
        }

        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("URL:");
                ui.add_sized([500.0, 22.0], TextEdit::singleline(&mut self.url));
                egui::ComboBox::from_label("")
                    .selected_text(match self.method { Method::GET=>"GET",Method::POST=>"POST",Method::PUT=>"PUT",Method::PATCH=>"PATCH",Method::DELETE=>"DELETE" })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.method, Method::GET, "GET");
                        ui.selectable_value(&mut self.method, Method::POST, "POST");
                        ui.selectable_value(&mut self.method, Method::PUT, "PUT");
                        ui.selectable_value(&mut self.method, Method::PATCH, "PATCH");
                        ui.selectable_value(&mut self.method, Method::DELETE, "DELETE");
                    });
                if ui.button(RichText::new("Send").strong()).clicked() {
                    self.send_current_request();
                    self.last_poll_instant = Some(Instant::now());
                }
                ui.checkbox(&mut self.compact_ui, "Compact");
                // Window float toggle button
                let float_label = if self.always_on_top { "Unfloat" } else { "Float" };
                if ui.button(float_label).on_hover_text("Toggle always-on-top window").clicked() {
                    self.always_on_top = !self.always_on_top;
                    ctx.send_viewport_cmd(egui::viewport::ViewportCommand::WindowLevel(
                        if self.always_on_top { egui::viewport::WindowLevel::AlwaysOnTop } else { egui::viewport::WindowLevel::Normal }
                    ));
                }
                ui.checkbox(&mut self.auto_poll, "Auto");
                ui.add(egui::Slider::new(&mut self.poll_interval_sec, 1.0..=120.0).text("sec"));
                if let Some(ms) = self.elapsed_ms {
                                    let code_opt = self.status_line.split_whitespace().nth(1).and_then(|s| s.parse::<u16>().ok());
                                    let color = match code_opt {
                                        Some(200) => Color32::from_rgb(0, 160, 0),       // green for 200
                                        Some(c) if c >= 400 => Color32::from_rgb(200, 0, 0), // red for 4xx/5xx
                                        Some(_) => Color32::from_rgb(200, 160, 0),       // yellow/orange for other non-200 (e.g., 201, 204, 3xx)
                                        None => Color32::WHITE,
                                    };
                                    ui.label(RichText::new(format!("{} | {} ms", self.status_line, ms)).color(color).strong());
                                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::SidePanel::left("left").resizable(true).min_width(200.0).default_width(360.0).show_inside(ui, |ui| {
                ui.heading("Request");
                egui::ScrollArea::vertical().auto_shrink([false;2]).show(ui, |ui| {
                ui.collapsing("Auth", |ui| { self.ui_auth(ui); });
                ui.collapsing("Headers", |ui| {
                    ui.add_sized([ui.available_width(), 100.0], TextEdit::multiline(&mut self.headers_text).code_editor());
                });
                match self.method {
                    Method::GET | Method::DELETE => { ui.label("Body (unused for GET/DELETE)"); }
                    _ => {
                        let prev_mode = self.body_mode;
                        ui.horizontal(|ui| {
                            ui.label("Body:");
                            ui.selectable_value(&mut self.body_mode, BodyMode::Json, "JSON");
                            ui.selectable_value(&mut self.body_mode, BodyMode::Form, "Form key=value");
                        });
                        if self.body_mode != prev_mode {
                            match self.body_mode {
                                BodyMode::Form => { self.update_form_from_body(); }
                                BodyMode::Json => { self.update_body_from_form(); }
                            }
                        }
                        match self.body_mode {
                            BodyMode::Json => {
                                ui.add_sized([ui.available_width(), 120.0], TextEdit::multiline(&mut self.body_text).code_editor());
                            }
                            BodyMode::Form => {
                                self.ui_form_editor(ui);
                            }
                        }
                    }
                }

                ui.separator();
                // Profiles Explorer (folder-like Projects -> Profiles)
                self.ui_profiles_panel(ui);

                ui.collapsing("cURL Import", |ui| {
                    ui.add_sized([ui.available_width(), 80.0], TextEdit::multiline(&mut self.curl_input));
                    if ui.button("Parse & Apply").clicked() { self.apply_curl(); }
                });
                });
            });

            egui::CentralPanel::default().show_inside(ui, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.resp_tab, 0, "Tree");
                        ui.selectable_value(&mut self.resp_tab, 1, "Raw");
                        if self.response.is_some() { ui.label(format!("size: {} bytes", self.raw_response.len())); }
                        if ui.button("Export").clicked() { self.export_dialog(); }
                    });

                    ui.separator();

                    match self.resp_tab {
                        0 => self.ui_json_tree(ui),
                        1 => self.ui_raw(ui),
                        _ => {}
                    }
                });
            });
        });
    }
}

impl AppState {
    fn send_current_request(&self) {
        let mut headers = data::build_headers(&self.auth, &self.headers_text);
        let body = match self.method {
            Method::GET | Method::DELETE => None,
            _ => {
                match self.body_mode {
                    BodyMode::Json => Some(self.body_text.clone()),
                    BodyMode::Form => {
                        let mut serializer = url::form_urlencoded::Serializer::new(String::new());
                        for f in &self.form_fields {
                            if f.enabled && !f.name.trim().is_empty() {
                                serializer.append_pair(f.name.trim(), &f.value);
                            }
                        }
                        let encoded = serializer.finish();
                        if !headers.iter().any(|(k, _)| k.eq_ignore_ascii_case("Content-Type")) {
                            headers.push(("Content-Type".into(), "application/x-www-form-urlencoded".into()));
                        }
                        Some(encoded)
                    }
                }
            }
        };
        let _ = self.tx.send(UserAction::SendRequest {
            url: self.url.clone(),
            method: self.method,
            headers,
            body,
        });
    }

    fn ui_raw(&self, ui: &mut egui::Ui) {
        ui.add_sized([ui.available_width(), ui.available_height()], TextEdit::multiline(&mut self.raw_response.as_str()).code_editor().desired_rows(30));
    }

    fn ui_diff(&mut self, ui: &mut egui::Ui) {
        if let Some(patch) = &self.json_patch {
            ui.label(RichText::new("JSON Patch (structural)").strong());
            ui.add_sized([ui.available_width(), 120.0], TextEdit::multiline(&mut patch.as_str()).code_editor());
        } else { ui.label("JSON Patch: <no previous or unparsable>"); }
        ui.separator();
        ui.label(RichText::new("Pretty JSON Text Diff").strong());
        if let Some(diff) = &self.text_diff {
            ui.add_sized([ui.available_width(), 200.0], TextEdit::multiline(&mut diff.as_str()).code_editor());
        } else { ui.label("<diff unavailable>"); }
    }

    fn ui_query(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.add(TextEdit::singleline(&mut self.search_text));
            if ui.button("Find").clicked() { self.update_search_results(); }
        });
        egui::ScrollArea::vertical().max_height(100.0).show(ui, |ui| {
            for line in &self.search_results { ui.label(line); }
        });

        ui.separator();
        ui.horizontal(|ui| { ui.label("JSONPath:"); ui.add(TextEdit::singleline(&mut self.jsonpath_query)); if ui.button("Run").clicked() { self.run_jsonpath(); } });
        ui.horizontal(|ui| { ui.label("JMESPath:"); ui.add(TextEdit::singleline(&mut self.jmespath_query)); if ui.button("Run").clicked() { self.run_jmespath(); } });
    }

    fn ui_json_tree(&mut self, ui: &mut egui::Ui) {
        // Query controls (results apply directly to Tree)
        ui.collapsing("Query", |ui| { self.ui_query(ui); });
        ui.separator();
        // Decide which JSON to render (filtered or full). Filtering is always enabled.
        let mut filtered: Option<Value> = None;
        if let Ok(v) = serde_json::from_str::<Value>(&self.jsonpath_result) { filtered = Some(v); }
        else if let Ok(v) = serde_json::from_str::<Value>(&self.jmespath_result) { filtered = Some(v); }
        egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
            if let Some(v) = filtered.as_ref() {
                render_value(ui, "root", v, "", true, &self.changed_ops, &self.prev_values);
            } else {
                match &self.response {
                    Some(v) => render_value(ui, "root", v, "", true, &self.changed_ops, &self.prev_values),
                    None => { ui.label("<no JSON or parse failed>"); }
                }
            }
        });
    }

    fn update_diffs(&mut self) {
        let (json_patch, text_diff) = data::compute_diffs(self.prev_response.as_ref(), self.response.as_ref());
        self.json_patch = json_patch;
        self.text_diff = text_diff;
        // Build changed_ops map from JSON Patch to support inline diff highlighting
        self.changed_ops.clear();
        if let Some(patch_str) = &self.json_patch {
            if let Ok(v) = serde_json::from_str::<Value>(patch_str) {
                if let Some(arr) = v.as_array() {
                    for item in arr {
                        if let (Some(op), Some(path)) = (item.get("op").and_then(|x| x.as_str()), item.get("path").and_then(|x| x.as_str())) {
                            self.changed_ops.insert(path.to_string(), op.to_string());
                        }
                    }
                }
            }
        }
        // Capture previous values for changed paths
        self.prev_values.clear();
        if let Some(prev) = self.prev_response.as_ref() {
            for (path, _op) in self.changed_ops.iter() {
                if let Some(val) = prev.pointer(path) {
                    self.prev_values.insert(path.clone(), val.clone());
                }
            }
        }
    }

    fn ui_auth(&mut self, ui: &mut egui::Ui) {
        egui::ComboBox::from_label("Type")
            .selected_text(match self.auth.kind { AuthType::None=>"None", AuthType::Basic=>"Basic", AuthType::Bearer=>"Bearer", AuthType::OAuth2Token=>"OAuth2 Token" })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.auth.kind, AuthType::None, "None");
                ui.selectable_value(&mut self.auth.kind, AuthType::Basic, "Basic");
                ui.selectable_value(&mut self.auth.kind, AuthType::Bearer, "Bearer");
                ui.selectable_value(&mut self.auth.kind, AuthType::OAuth2Token, "OAuth2 Token");
            });
        match self.auth.kind {
            AuthType::None => {}
            AuthType::Basic => {
                ui.horizontal(|ui| {
                    ui.label("User:"); ui.add(TextEdit::singleline(&mut self.auth.username));
                    ui.label("Pass:"); ui.add(TextEdit::singleline(&mut self.auth.password));
                });
            }
            AuthType::Bearer => { ui.add(TextEdit::singleline(&mut self.auth.bearer_token).hint_text("token")); }
            AuthType::OAuth2Token => { ui.add(TextEdit::singleline(&mut self.auth.oauth2_access_token).hint_text("access_token")); }
        }
    }

    fn ui_profiles_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Profiles");
        ui.horizontal_wrapped(|ui| {
            ui.label("New Project:");
            ui.add(TextEdit::singleline(&mut self.new_project_name).hint_text("name"));
            if ui.button("Add").clicked() {
                if !self.new_project_name.trim().is_empty() {
                    let _ = data::create_project(&self.new_project_name);
                    self.current_project = self.new_project_name.clone();
                    self.new_project_name.clear();
                    self.refresh_projects();
                }
            }
            if ui.button("Refresh").clicked() { self.refresh_projects(); }
        });
        egui::ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
            let projects = data::list_projects().unwrap_or_default();
            for p in projects {
                let header = format!("📁 {}", p);
                egui::CollapsingHeader::new(RichText::new(header).strong())
                    .id_source(format!("proj_hdr_{}", p))
                    .default_open(self.current_project == p)
                    .show(ui, |ui| {
                        let profiles = data::list_profiles(&p).unwrap_or_default();
                        for name in profiles {
                            let selected = self.current_project == p && self.selected_profile == name;
                            let resp = ui.selectable_label(selected, format!("📄 {}", name));
                            if resp.clicked() {
                                self.current_project = p.clone();
                                self.selected_profile = name.clone();
                            }
                            if resp.double_clicked() {
                                self.current_project = p.clone();
                                self.selected_profile = name.clone();
                                self.load_profile_clicked();
                            }
                        }
                        // Inline new profile creation for this project
                        if self.current_project == p {
                            ui.horizontal(|ui| {
                                ui.label("New profile:");
                                ui.add(TextEdit::singleline(&mut self.new_profile_name).hint_text("name"));
                                if ui.button("Save as").clicked() {
                                    if !self.new_profile_name.trim().is_empty() {
                                        self.profile_name = self.new_profile_name.clone();
                                        self.save_profile_clicked();
                                        self.new_profile_name.clear();
                                    }
                                }
                            });
                        }
                    });
            }
        });
        ui.separator();
        ui.label(format!("Selected project: {}", self.current_project));
        ui.horizontal_wrapped(|ui| {
            ui.label("Rename project:");
            ui.add(TextEdit::singleline(&mut self.rename_project_name).hint_text("new name"));
            if ui.button("Rename").clicked() {
                if !self.rename_project_name.trim().is_empty() {
                    let old = self.current_project.clone();
                    let _ = data::rename_project(&old, &self.rename_project_name);
                    self.current_project = self.rename_project_name.clone();
                    self.rename_project_name.clear();
                    self.refresh_projects();
                }
            }
            if ui.button("Delete Project").clicked() {
                let old = self.current_project.clone();
                let _ = data::delete_project(&old);
                self.current_project = "default".to_string();
                let _ = data::create_project(&self.current_project);
                self.refresh_projects();
            }
        });
        ui.horizontal_wrapped(|ui| {
            ui.label("Selected profile:");
            ui.monospace(self.selected_profile.clone());
            if ui.button("Load").clicked() { self.load_profile_clicked(); }
            if ui.button("Save").clicked() {
                if !self.selected_profile.is_empty() {
                    self.profile_name = self.selected_profile.clone();
                    self.save_profile_clicked();
                }
            }
            if ui.button("Delete").clicked() { self.delete_profile_clicked(); }
        });
        ui.horizontal_wrapped(|ui| {
            ui.label("Rename profile:");
            ui.add(TextEdit::singleline(&mut self.rename_profile_name).hint_text("new name"));
            if ui.button("Rename").clicked() {
                if !self.selected_profile.is_empty() && !self.rename_profile_name.trim().is_empty() {
                    let old = self.selected_profile.clone();
                    let new = self.rename_profile_name.clone();
                    let _ = data::rename_profile(&self.current_project, &old, &new);
                    self.selected_profile = new;
                    self.rename_profile_name.clear();
                    self.refresh_projects();
                }
            }
        });
    }

    fn save_profile_clicked(&mut self) {
        // Determine profile name
        if self.profile_name.trim().is_empty() {
            self.profile_name = format!("{} {}", self.method_string(), data::chrono_like_now());
        }
        let name = self.profile_name.clone();
        // Ensure body_text reflects current form as JSON for consistency
        if let BodyMode::Form = self.body_mode { self.update_body_from_form(); }
        let saved_body = self.body_text.clone();
        let prof = RequestProfile {
            name: name.clone(),
            url: self.url.clone(),
            method: self.method,
            headers_text: self.headers_text.clone(),
            body_text: saved_body,
            auth: self.auth.clone(),
        };
        let _ = data::save_profile_to_project(&self.current_project, &prof);
        self.selected_profile = name;
        self.refresh_projects();
    }

    fn load_profile_clicked(&mut self) {
        // Prefer selected_profile; fallback to profile_name
        let name = if !self.selected_profile.is_empty() { self.selected_profile.clone() } else { self.profile_name.clone() };
        if name.trim().is_empty() { return; }
        if let Ok(p) = data::load_profile_from_project(&self.current_project, &name) {
            self.apply_profile(p);
        }
    }

    fn delete_profile_clicked(&mut self) {
        let name = if !self.selected_profile.is_empty() { self.selected_profile.clone() } else { self.profile_name.clone() };
        if name.trim().is_empty() { return; }
        let _ = data::delete_profile_from_project(&self.current_project, &name);
        if self.selected_profile == name { self.selected_profile.clear(); }
        self.refresh_projects();
    }

    fn manage_profiles_dialog(&mut self) {
        // For now simply refresh lists; a full dialog can be added later
        self.refresh_projects();
    }

    fn apply_profile(&mut self, p: RequestProfile) {
        self.url = p.url; self.method = p.method; self.headers_text = p.headers_text; self.body_text = p.body_text; self.auth = p.auth; self.profile_name = p.name.clone();
        // Sync form fields from loaded body
        self.update_form_from_body();
        // Update selected profile to the applied one
        self.selected_profile = self.profile_name.clone();
    }

    fn load_profiles(&mut self) {
        // Load legacy profiles (if any) for migration/compat; not used by new project system
        self.profiles = data::load_profiles_from_legacy().unwrap_or_default();
    }

    fn refresh_projects(&mut self) {
        // List projects; always include a convenient "default" entry
        let mut projs = data::list_projects().unwrap_or_default();
        if !projs.iter().any(|p| p == "default") { projs.insert(0, "default".into()); }
        // Ensure current_project is valid (keep custom value even if dir not yet created)
        if !projs.iter().any(|p| p == &self.current_project) { projs.insert(0, self.current_project.clone()); }
        projs.sort(); projs.dedup();
        self.available_projects = projs;

        // Load profiles for current project
        self.available_profiles = data::list_profiles(&self.current_project).unwrap_or_default();
        // Maintain selection if possible
        if !self.available_profiles.iter().any(|n| n == &self.selected_profile) {
            self.selected_profile = self.available_profiles.first().cloned().unwrap_or_default();
        }
    }

    fn apply_curl(&mut self) {
        if let Some(parsed) = data::parse_curl(&self.curl_input) {
            self.url = parsed.url;
            self.method = parsed.method;
            if !parsed.headers.is_empty() {
                self.headers_text = parsed.headers.iter().map(|(k,v)| format!("{}: {}", k, v)).collect::<Vec<_>>().join("\n");
            }
            if let Some(b) = parsed.body { self.body_text = b; }
            // Sync form fields with the parsed body
            self.update_form_from_body();
        }
    }

    fn update_search_results(&mut self) {
        self.search_results.clear();
        if let Some(v) = &self.response {
            let pretty = serde_json::to_string_pretty(v).unwrap_or_default();
            self.search_results = data::lines_containing(&pretty, &self.search_text);
        }
    }

    fn run_jsonpath(&mut self) {
        self.jsonpath_result.clear();
        if self.jsonpath_query.trim().is_empty() { return; }
        if let Some(v) = &self.response {
            match jsonpath_lib::select(v, &self.jsonpath_query) {
                Ok(nodes) => {
                    let arr: Vec<&Value> = nodes;
                    self.jsonpath_result = serde_json::to_string_pretty(&arr).unwrap_or_default();
                }
                Err(e) => { self.jsonpath_result = format!("error: {}", e); }
            }
        }
    }

    fn run_jmespath(&mut self) {
        self.jmespath_result.clear();
        if self.jmespath_query.trim().is_empty() { return; }
        if let Some(v) = &self.response {
            let s = serde_json::to_string(v).unwrap_or_else(|_| "null".to_string());
            let data_var = match jmespath::Variable::from_json(&s) {
                Ok(d) => d,
                Err(e) => { self.jmespath_result = format!("parse error: {}", e); return; }
            };
            match jmespath::compile(&self.jmespath_query) {
                Ok(expr) => match expr.search(data_var) {
                    Ok(res) => { self.jmespath_result = serde_json::to_string_pretty(&res).unwrap_or_else(|_| format!("{}", res)); }
                    Err(e) => { self.jmespath_result = format!("error: {}", e); }
                }
                Err(e) => { self.jmespath_result = format!("compile error: {}", e); }
            }
        }
    }

    fn export_dialog(&mut self) {
        if self.response.is_none() { return; }
        if let Some(path) = rfd::FileDialog::new().set_title("Export JSON").save_file() {
            let _ = std::fs::write(&path, &self.raw_response);
        }
        if let Some(Value::Array(arr)) = &self.response {
            if let Some(path) = rfd::FileDialog::new().set_title("Export NDJSON").save_file() {
                let mut out = String::new();
                for item in arr { out.push_str(&serde_json::to_string(item).unwrap_or("{}".into())); out.push('\n'); }
                let _ = std::fs::write(&path, out);
            }
            if let Some(path) = rfd::FileDialog::new().set_title("Export CSV").save_file() {
                let mut wtr = csv::Writer::from_path(&path).ok();
                if let Some(w) = wtr.as_mut() {
                    let mut headers: Vec<String> = vec![];
                    let rows: Vec<std::collections::BTreeMap<String, String>> = arr.iter().map(|v| data::flatten_json_to_row(v)).collect();
                    for row in &rows { for k in row.keys() { if !headers.contains(k) { headers.push(k.clone()); } } }
                    let _ = w.write_record(headers.iter());
                    for row in rows {
                        let record: Vec<String> = headers.iter().map(|h| row.get(h).cloned().unwrap_or_default()).collect();
                        let _ = w.write_record(&record);
                    }
                    let _ = w.flush();
                }
            }
        }
    }

    fn method_string(&self) -> String { match self.method { Method::GET=>"GET".into(), Method::POST=>"POST".into(), Method::PUT=>"PUT".into(), Method::PATCH=>"PATCH".into(), Method::DELETE=>"DELETE".into() } }
}

// ===== Rendering helpers (UI-only) =====
fn render_value(ui: &mut egui::Ui, label: &str, v: &Value, path: &str, diff_inline: bool, changed_ops: &std::collections::HashMap<String, String>, prev_values: &std::collections::HashMap<String, Value>) {
    // Helper to decide label style based on inline diff
    let mut label_rt = RichText::new(label).strong();
    if diff_inline {
        if let Some(op) = changed_ops.get(path) {
            label_rt = label_rt.color(op_color(op));
        } else if has_descendant_change(path, changed_ops) {
            // subtle hint on containers that have changes somewhere below
            label_rt = label_rt.italics();
        }
    }

    match v {
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {
            ui.horizontal(|ui| {
                ui.label(label_rt);
                ui.label(brief(v));
                if diff_inline {
                    if let Some(op) = changed_ops.get(path) {
                        if op == "replace" {
                            if let Some(prev_v) = prev_values.get(path) {
                                let old_str = brief_json_value_str(prev_v);
                                ui.label(RichText::new(format!(" (old: {})", old_str)).color(Color32::GRAY));
                            }
                        }
                    }
                }
            });
        }
        Value::Array(arr) => {
            let header = format!("{} [Array] ({} items)", label, arr.len());
            let mut header_rt = RichText::new(header).strong();
            if diff_inline {
                if let Some(op) = changed_ops.get(path) { header_rt = header_rt.color(op_color(op)); }
                else if has_descendant_change(path, changed_ops) { header_rt = header_rt.italics(); }
            }
            egui::CollapsingHeader::new(header_rt)
                .default_open(false)
                .show(ui, |ui| {
                    egui::ScrollArea::vertical().auto_shrink([false;2]).show(ui, |ui| {
                        for (i, item) in arr.iter().enumerate() {
                            let child_path = if path.is_empty() { format!("/{}", i) } else { format!("{}/{}", path, i) };
                            render_value(ui, &format!("[{}]", i), item, &child_path, diff_inline, changed_ops, prev_values);
                        }
                    });
                });
        }
        Value::Object(map) => {
            let header = format!("{} {{Object}} ({} keys)", label, map.len());
            let mut header_rt = RichText::new(header).strong();
            if diff_inline {
                if let Some(op) = changed_ops.get(path) { header_rt = header_rt.color(op_color(op)); }
                else if has_descendant_change(path, changed_ops) { header_rt = header_rt.italics(); }
            }
            egui::CollapsingHeader::new(header_rt)
                .default_open(true)
                .show(ui, |ui| {
                    for (k, val) in map.iter() {
                        let seg = escape_pointer_segment(k);
                        let child_path = if path.is_empty() { format!("/{}", seg) } else { format!("{}/{}", path, seg) };
                        render_value(ui, k, val, &child_path, diff_inline, changed_ops, prev_values);
                    }
                });
        }
    }
}

fn escape_pointer_segment(s: &str) -> String {
    s.replace("~", "~0").replace("/", "~1")
}

fn has_descendant_change(path: &str, changed_ops: &std::collections::HashMap<String, String>) -> bool {
    let prefix = if path.is_empty() { "/".to_string() } else { format!("{}/", path) };
    changed_ops.keys().any(|p| p.starts_with(&prefix))
}

fn op_color(op: &str) -> Color32 {
    match op {
        "add" => Color32::from_rgb(0, 160, 0),
        "remove" => Color32::from_rgb(200, 0, 0),
        "replace" => Color32::from_rgb(200, 160, 0),
        "move" => Color32::LIGHT_BLUE,
        "copy" => Color32::LIGHT_BLUE,
        _ => Color32::LIGHT_BLUE,
    }
}

fn brief_json_value_str(v: &Value) -> String {
    match v {
        Value::Null => "null".into(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => {
            let mut sref = s.as_str();
            if sref.len() > 120 { sref = &sref[..120]; }
            format!("\"{}\"", sref)
        }
        _ => "...".into(),
    }
}

fn brief(v: &Value) -> RichText {
    match v {
        Value::Null => RichText::new("null").color(Color32::GRAY),
        Value::Bool(b) => RichText::new(format!("{}", b)).color(Color32::from_rgb(0, 160, 0)),
        Value::Number(n) => RichText::new(n.to_string()).color(Color32::from_rgb(0, 120, 200)),
        Value::String(s) => {
            let mut s = s.as_str();
            if s.len() > 120 { s = &s[..120]; }
            RichText::new(format!("\"{}\"", s)).color(Color32::from_rgb(160, 100, 0))
        }
        _ => RichText::new("...")
    }
}

pub fn start_app() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions { ..Default::default() };
    eframe::run_native(
        "API Tester",
        native_options,
        Box::new(|_cc| Box::new(AppState::default())),
    )
}


impl AppState {
    fn ui_form_editor(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("+ Add").clicked() {
                self.form_fields.push(FormField { enabled: true, name: String::new(), value: String::new() });
            }
            if ui.button("Clear").clicked() {
                self.form_fields.clear();
            }
        });
        for i in 0..self.form_fields.len() {
            let mut to_remove = false;
            let row = &mut self.form_fields[i];
            ui.horizontal(|ui| {
                ui.checkbox(&mut row.enabled, "");
                ui.add_sized([150.0, 22.0], TextEdit::singleline(&mut row.name).hint_text("name"));
                ui.label("=");
                let avail = ui.available_width() - 40.0; // leave some room for the delete button
                let w = if avail > 160.0 { avail } else { 160.0 };
                ui.add_sized([w, 22.0], TextEdit::singleline(&mut row.value).hint_text("value"));
                if ui.button("✕").clicked() { to_remove = true; }
            });
            if to_remove { self.form_fields.remove(i); break; }
        }
        if self.form_fields.is_empty() {
            ui.label(RichText::new("No fields. Click + Add to insert a row.").italics().color(Color32::GRAY));
        }
        ui.label(RichText::new("Will send as application/x-www-form-urlencoded").small().color(Color32::GRAY));
        // Keep Body JSON in sync with current form fields
        self.update_body_from_form();
    }
}

impl AppState {
    fn update_form_from_body(&mut self) {
        // Try JSON first
        let txt = self.body_text.trim();
        let mut rows: Vec<FormField> = Vec::new();
        if !txt.is_empty() {
            if let Ok(v) = serde_json::from_str::<Value>(txt) {
                match v {
                    Value::Object(map) => {
                        for (k, val) in map {
                            match val {
                                Value::Array(arr) => {
                                    // Expand arrays as repeated keys (stringify elements)
                                    for elem in arr {
                                        let vstr = match &elem {
                                            Value::String(s) => s.clone(),
                                            Value::Number(n) => n.to_string(),
                                            Value::Bool(b) => b.to_string(),
                                            Value::Null => String::new(),
                                            _ => serde_json::to_string(&elem).unwrap_or_default(),
                                        };
                                        rows.push(FormField{ enabled: true, name: k.clone(), value: vstr });
                                    }
                                }
                                other => {
                                    let vstr = match &other {
                                        Value::String(s) => s.clone(),
                                        Value::Number(n) => n.to_string(),
                                        Value::Bool(b) => b.to_string(),
                                        Value::Null => String::new(),
                                        _ => serde_json::to_string(&other).unwrap_or_default(),
                                    };
                                    rows.push(FormField{ enabled: true, name: k.clone(), value: vstr });
                                }
                            }
                        }
                    }
                    _ => {
                        // Non-object JSON not suitable for form pairs; fall back to URL-encoded parsing
                    }
                }
            }
            if rows.is_empty() {
                // Try URL-encoded key=value pairs
                for (k, v) in url::form_urlencoded::parse(txt.as_bytes()) {
                    rows.push(FormField{ enabled: true, name: k.to_string(), value: v.to_string() });
                }
            }
        }
        if rows.is_empty() {
            self.form_fields.clear();
            self.form_fields.push(FormField { enabled: true, name: String::new(), value: String::new() });
        } else {
            self.form_fields = rows;
        }
    }

    fn update_body_from_form(&mut self) {
        use std::collections::BTreeMap;
        let mut multimap: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for f in &self.form_fields {
            let name = f.name.trim();
            if f.enabled && !name.is_empty() {
                multimap.entry(name.to_string()).or_default().push(f.value.clone());
            }
        }
        let mut obj = serde_json::Map::new();
        for (k, vals) in multimap.into_iter() {
            if vals.len() == 1 {
                obj.insert(k, Value::String(vals[0].clone()));
            } else {
                obj.insert(k, Value::Array(vals.into_iter().map(Value::String).collect()));
            }
        }
        let v = Value::Object(obj);
        self.body_text = serde_json::to_string_pretty(&v).unwrap_or_else(|_| "{}".to_string());
    }
}

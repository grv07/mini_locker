use ansi_term::{Colour::Fixed, Style};
use std::collections::{BTreeMap, HashMap};
use zellij_tile::prelude::*;

type PaneModeStore = HashMap<u32, InputMode>;
type TabModeStore = (InputMode, PaneModeStore);
type ModeStore = HashMap<usize, TabModeStore>;

#[derive(Default, Debug)]
struct ModeState {
    store: ModeStore,
}

// We can first heck the tab mode later we check indiviual pane mode so if mode is not set for the pane it will inherite from tab
// other wise use its own.

impl ModeState {
    fn set_tab_mode(&mut self, pos: usize, mode: InputMode) -> Option<TabModeStore> {
        self.store.insert(pos, (mode, HashMap::new()))
    }

    fn set_pane_mode(&mut self, pos: usize, id: u32, mode: InputMode) {
        let tab_store = self.store.get_mut(&pos);

        // If TabStore is there
        if let Some((_, tab_store)) = tab_store {
            tab_store.insert(id, mode);
        } else {
            let mut panes = HashMap::new();
            panes.insert(id, mode);

            self.store.insert(pos, (InputMode::Normal, panes));
        }
    }

    fn remove_tab_mode(&mut self, pos: usize) {
        self.set_tab_mode(pos, InputMode::Normal);
    }

    fn remove_pane_mode(&mut self, pos: usize, id: u32) {
        let pane_store = self.store.get_mut(&pos);

        // If TabStore is there
        if let Some((_, pane_store)) = pane_store {
            let _ = pane_store.remove(&id);
        }
    }

    fn get_pane_mode(&self, pos: &usize, id: u32) -> InputMode {
        let pane_store = self.store.get(pos);

        // If TabState is there
        if let Some((_, pane_store)) = pane_store {
            return pane_store.get(&id).unwrap_or(&InputMode::Normal).clone();
        } else {
            return InputMode::Normal;
        }
    }

    fn get_tab_mode(&self, pos: &usize) -> InputMode {
        let default = (InputMode::Normal, HashMap::new());
        let (mode, _) = self.store.get(&pos).unwrap_or(&default);

        mode.clone()
    }
}

#[derive(Default)]
struct State {
    active_tab: usize,
    // set_mode_to: Option<InputMode>,
    mode_state: ModeState,
    panes_manifest: PaneManifest,
    active_item: usize,
    items_count: usize,
    selected_ids: (usize, Option<u32>),
    // selected_pane_id: u32,
    userspace_configuration: BTreeMap<String, String>,
}

impl State {
    fn to_mode_str(mode: InputMode) -> &'static str {
        match mode {
            InputMode::Locked => "L",
            _ => "N",
        }
    }

    fn print_tab_item(&mut self, pos: &usize, m: &str) {
        let tab_title = format!("Tab #{} M: {m}", pos + 1);
        if self.items_count == self.active_item {
            self.selected_ids = (*pos, None);
            let tab_title = color_bold(GREEN, &tab_title);
            println!("{}", tab_title);
        } else {
            let tab_title = color_bold(CYAN, &tab_title);
            println!("{}", tab_title);
        }
    }

    fn print_pane_item(&mut self, pos: &usize, pane: &PaneInfo, m: &str) {
        if self.items_count == self.active_item {
            self.selected_ids = (*pos, Some(pane.id));
            let pane_title = color_bold(ORANGE, &pane.title);
            println!("  {} M: {}", pane_title, m);
        } else {
            let pane_title = color_bold(CYAN, &pane.title);

            println!("  {}", pane_title);
        }
    }

    fn get_active_tab_panes_len(&mut self) -> usize {
        self.panes_manifest
            .panes
            .get(&self.active_tab)
            .unwrap_or(&vec![])
            .iter()
            .filter(|p| !p.is_plugin)
            .collect::<Vec<_>>()
            .len()
    }

    fn get_focused_pane_id(&self) -> Option<u32> {
        let default = &vec![];
        let panes = self
            .panes_manifest
            .panes
            .get(&self.active_tab)
            .unwrap_or(default);

        for pane in panes {
            if pane.is_focused {
                return Some(pane.id);
            }
        }

        None
    }
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, configuration: BTreeMap<String, String>) {
        self.userspace_configuration = configuration;
        // we need the ReadApplicationState permission to receive the ModeUpdate and TabUpdate
        // events
        // we need the RunCommands permission to run "cargo test" in a floating window
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::RunCommands,
            PermissionType::ChangeApplicationState,
        ]);
        subscribe(&[
            // EventType::ModeUpdate,
            EventType::PaneUpdate,
            EventType::TabUpdate,
            EventType::Key,
        ]);
    }

    fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
        eprintln!("pipe_message: {:?}", pipe_message);
        true
    }

    fn update(&mut self, event: Event) -> bool {
        let mut should_render = false;
        match event {
            Event::TabUpdate(tab_info) => {
                let tab_info = tab_info.iter().find(|t| t.active);
                if let Some(tab) = tab_info {
                    self.active_tab = tab.position;

                    let mode = self.mode_state.get_tab_mode(&self.active_tab);
                    switch_to_input_mode(&mode);
                }

                should_render = true;
            }

            Event::PaneUpdate(pane_info) => {
                self.panes_manifest = pane_info;

                if let Some(p_id) = self.get_focused_pane_id() {
                    let mode = self.mode_state.get_pane_mode(&self.active_tab, p_id);

                    switch_to_input_mode(&mode);
                }
                should_render = true;
            }

            Event::Key(Key::Char('j')) => {
                self.active_item = (self.active_item + 1) % self.items_count;
                should_render = true;
            }

            Event::Key(Key::Char('k')) => {
                self.active_item = (self.active_item + (self.items_count - 1)) % self.items_count;
                should_render = true;
            }

            Event::Key(Key::Char('L')) => {
                let (pos, id) = self.selected_ids;

                if let Some(id) = id {
                    self.mode_state.set_pane_mode(pos, id, InputMode::Locked);
                } else {
                    self.mode_state.set_tab_mode(pos, InputMode::Locked);
                }

                should_render = true;
            }

            Event::Key(Key::Char('N')) => {
                let (pos, id) = self.selected_ids;

                if let Some(id) = id {
                    self.mode_state.set_pane_mode(pos, id, InputMode::Normal);
                } else {
                    self.mode_state.set_tab_mode(pos, InputMode::Normal);
                }

                should_render = true;
            }

            Event::Key(Key::Char('\n')) => {
                let (pos, id) = self.selected_ids;
                if let Some(id) = id {
                    focus_terminal_pane(id, true);
                } else {
                    go_to_tab(pos as u32);
                }
                should_render = true;
            }
            _ => (),
        };

        should_render
    }

    fn render(&mut self, _rows: usize, _cols: usize) {
        self.items_count = 0;

        let panes = self.panes_manifest.panes.clone();
        //render the current active tab
        if let Some((pos, panes)) = panes.get_key_value(&self.active_tab) {
            let m = Self::to_mode_str(self.mode_state.get_tab_mode(pos));
            self.print_tab_item(pos, m);

            for pane in panes.iter().filter(|p| !p.is_plugin && !p.is_floating) {
                self.items_count += 1;
                let m = Self::to_mode_str(self.mode_state.get_pane_mode(pos, pane.id));
                self.print_pane_item(pos, pane, m);
            }
        }

        for (pos, panes) in panes.iter() {
            if pos == &self.active_tab {
                continue;
            }

            self.items_count += 1;
            let m = Self::to_mode_str(self.mode_state.get_tab_mode(pos));
            self.print_tab_item(pos, m);

            for pane in panes.iter().filter(|p| !p.is_plugin && !p.is_floating) {
                self.items_count += 1;
                let m = Self::to_mode_str(self.mode_state.get_pane_mode(&pos, pane.id));
                self.print_pane_item(&pos, pane, m);
            }
            self.items_count += 1;
        }
    }
}

pub const CYAN: u8 = 51;
pub const GRAY_LIGHT: u8 = 238;
pub const GRAY_DARK: u8 = 245;
pub const WHITE: u8 = 15;
pub const BLACK: u8 = 16;
pub const RED: u8 = 124;
pub const GREEN: u8 = 154;
pub const ORANGE: u8 = 166;

fn color_bold(color: u8, text: &str) -> String {
    format!("{}", Style::new().fg(Fixed(color)).bold().paint(text))
}

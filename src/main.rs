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

    fn get_pane_mode(&mut self, pos: usize, id: u32) -> InputMode {
        let pane_store = self.store.get_mut(&pos);

        // If TabState is there
        if let Some((_, pane_store)) = pane_store {
            return pane_store.get(&id).unwrap_or(&InputMode::Normal).clone();
        } else {
            return InputMode::Normal;
        }
    }

    fn get_tab_mode(&self, pos: usize) -> InputMode {
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
    selected_pane: usize,
    selected_pane_id: u32,
    userspace_configuration: BTreeMap<String, String>,
}

impl State {
    fn to_mode_str(mode: InputMode) -> &'static str {
        match mode {
            InputMode::Locked => "L",
            _ => "N",
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
                    should_render = true;
                }
            }

            Event::PaneUpdate(pane_info) => {
                self.panes_manifest = pane_info;

                if let Some(p_id) = self.get_focused_pane_id() {
                    let mode = self.mode_state.get_pane_mode(self.active_tab, p_id);

                    switch_to_input_mode(&mode);
                    should_render = true;
                }
            }

            Event::Key(Key::Char('j')) => {
                self.selected_pane = (self.selected_pane + 1) % self.get_active_tab_panes_len();
                should_render = true;
            }

            Event::Key(Key::Char('k')) => {
                self.selected_pane = (self.selected_pane + (self.get_active_tab_panes_len() - 1))
                    % self.get_active_tab_panes_len();
                should_render = true;
            }

            Event::Key(Key::Char('L')) => {
                // self.set_mode_to = Some(InputMode::Locked);
                self.mode_state.set_pane_mode(
                    self.active_tab,
                    self.selected_pane_id,
                    InputMode::Locked,
                );
                should_render = true;
            }

            Event::Key(Key::Char('N')) => {
                // self.set_mode_to = Some(InputMode::Normal);
                self.mode_state.set_pane_mode(
                    self.active_tab,
                    self.selected_pane_id,
                    InputMode::Normal,
                );
                should_render = true;
            }

            Event::Key(Key::Char('\n')) => {
                focus_terminal_pane(self.selected_pane_id, true);
                should_render = true;
            }
            _ => (),
        };

        should_render
    }

    fn render(&mut self, _rows: usize, _cols: usize) {
        for (i, pane) in self
            .panes_manifest
            .panes
            .get(&self.active_tab)
            .unwrap_or(&vec![])
            .iter()
            .enumerate()
        {
            if pane.is_plugin {
                continue;
            }

            if i == self.selected_pane {
                self.selected_pane_id = pane.id;
                let pane_title = color_bold(ORANGE, &pane.title);

                println!(
                    "> {} {} M: {}",
                    pane_title,
                    pane.id,
                    Self::to_mode_str(self.mode_state.get_pane_mode(self.active_tab, pane.id))
                );

                continue;
            }

            let pane_title = color_bold(CYAN, &pane.title);

            println!("{}", pane_title);
        }
        println!("");
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

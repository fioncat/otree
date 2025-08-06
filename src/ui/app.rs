use std::io::Stdout;
use std::rc::Rc;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{Event, KeyEvent, MouseButton, MouseEventKind};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::{Frame, Terminal};
use serde_json::Value;

use crate::clipboard::write_clipboard;
use crate::config::keys::Action;
use crate::config::{Config, LayoutDirection};
use crate::debug;
use crate::edit::Edit;
use crate::live_reload::FileWatcher;
use crate::tree::Tree;
use crate::ui::data_block::DataBlock;
use crate::ui::filter::Filter;
use crate::ui::footer::{Footer, FooterText};
use crate::ui::header::{Header, HeaderContext};
use crate::ui::popup::{Popup, PopupLevel};
use crate::ui::tree_overview::TreeOverview;

use super::filter::FilterAction;

enum Refresh {
    /// Update the TUI
    Update,
    /// Skip the update of the TUI
    Skip,
    /// Quit the TUI and return to the shell
    Quit,
    /// Quit the TUI and edit text
    Edit(Box<Edit>),
}

#[derive(Debug, Clone, Copy)]
enum ElementInFocus {
    TreeOverview,
    DataBlock,
    Popup,
    Filter,
    None,
}

pub enum ScrollDirection {
    Up,
    Down,
}

pub struct App {
    cfg: Rc<Config>,

    focus: ElementInFocus,
    last_focus: Option<ElementInFocus>,

    tree_overview: TreeOverview,
    tree_overview_area: Rect,

    filter: Option<Filter>,
    filter_area: Rect,

    data_block: DataBlock,
    data_block_area: Rect,

    layout_direction: LayoutDirection,
    layout_tree_size: u16,

    header: Option<Header>,
    header_area: Rect,
    skip_header: bool,

    footer: Option<Footer>,
    footer_area: Rect,
    skip_footer: bool,
    foot_message: Option<String>,

    popup: Popup,
    before_popup_focus: ElementInFocus,

    fw: Option<FileWatcher>,
}

pub(super) enum ShowResult {
    Edit(Box<Edit>),
    Quit,
}

impl App {
    const HEADER_HEIGHT: u16 = 1;
    const FOOTER_HEIGHT: u16 = 1;

    const FILTER_HEIGHT: u16 = 3;

    const POLL_EVENT_DURATION: Duration = Duration::from_millis(100);

    pub fn new(cfg: Rc<Config>, tree: Tree, fw: Option<FileWatcher>) -> Self {
        let footer = if cfg.footer.disable {
            None
        } else {
            Some(Footer::new(cfg.clone()))
        };
        Self {
            cfg: cfg.clone(),
            focus: ElementInFocus::TreeOverview,
            last_focus: None,
            tree_overview: TreeOverview::new(cfg.clone(), tree),
            tree_overview_area: Rect::default(),
            filter: None,
            filter_area: Rect::default(),
            data_block: DataBlock::new(cfg.clone()),
            data_block_area: Rect::default(),
            layout_direction: cfg.layout.direction,
            layout_tree_size: cfg.layout.tree_size,
            header: None,
            header_area: Rect::default(),
            skip_header: false,
            footer,
            footer_area: Rect::default(),
            skip_footer: false,
            foot_message: None,
            popup: Popup::new(cfg.clone()),
            before_popup_focus: ElementInFocus::None,
            fw,
        }
    }

    pub fn set_header(&mut self, ctx: HeaderContext) {
        self.header = Some(Header::new(self.cfg.clone(), ctx));
    }

    pub(super) fn show(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<ShowResult> {
        debug!("Start ui show loop, with config: {:?}", self.cfg);
        terminal.draw(|frame| self.draw(frame))?;

        loop {
            let refresh = if self.fw.is_some() {
                if crossterm::event::poll(Self::POLL_EVENT_DURATION)? {
                    self.refresh()?
                } else {
                    self.refresh_with_fw()?
                }
            } else {
                self.refresh()?
            };

            match refresh {
                Refresh::Update => {
                    terminal.draw(|frame| self.draw(frame))?;
                }
                Refresh::Skip => continue,
                Refresh::Edit(edit) => return Ok(ShowResult::Edit(edit)),
                Refresh::Quit => return Ok(ShowResult::Quit),
            }
        }
    }

    fn refresh(&mut self) -> Result<Refresh> {
        let refresh = match crossterm::event::read()? {
            Event::Key(key) => self.on_key(key),
            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::Down(MouseButton::Left) => self.on_click(mouse.column, mouse.row),
                MouseEventKind::ScrollUp => {
                    self.on_scroll(ScrollDirection::Up, mouse.column, mouse.row)
                }
                MouseEventKind::ScrollDown => {
                    self.on_scroll(ScrollDirection::Down, mouse.column, mouse.row)
                }
                _ => Refresh::Skip,
            },
            // When resize happens, we need to redraw the widgets to fit the new size
            Event::Resize(_, _) => Refresh::Update,
            Event::FocusGained => self.on_focus_changed(true),
            Event::FocusLost => self.on_focus_changed(false),
            _ => Refresh::Skip,
        };
        Ok(refresh)
    }

    fn refresh_with_fw(&mut self) -> Result<Refresh> {
        if crossterm::event::poll(Self::POLL_EVENT_DURATION)? {
            return self.refresh();
        }
        let fw = self.fw.as_ref().unwrap();

        if let Some(err_msg) = fw.get_err() {
            let msg = format!("Failed to parse file: {err_msg}");
            self.foot_message = Some(msg);
            return Ok(Refresh::Update);
        }

        let maybe_tree = match fw.parse_tree() {
            Ok(tree) => tree,
            Err(e) => {
                let message = format!("Failed to watch file events: {e:#}");
                self.popup(message, PopupLevel::Error);
                return Ok(Refresh::Update);
            }
        };

        match maybe_tree {
            Some(tree) => {
                self.reload_tree(tree);
                self.foot_message = Some(String::from("File updated, tree reloaded"));
                Ok(Refresh::Update)
            }
            None => Ok(Refresh::Skip),
        }
    }

    fn reload_tree(&mut self, tree: Tree) {
        // TODO: we can keep tree status
        self.focus = ElementInFocus::TreeOverview;
        self.last_focus = None;
        self.tree_overview = TreeOverview::new(self.cfg.clone(), tree);
        self.tree_overview_area = Rect::default();
        self.data_block = DataBlock::new(self.cfg.clone());
        self.data_block_area = Rect::default();
    }

    fn draw(&mut self, frame: &mut Frame) {
        self.refresh_area(frame);

        let selected = self.tree_overview.get_selected();
        if let Some(id) = selected {
            if let Some(item) = self.tree_overview.get_value(id.as_str()) {
                self.data_block.update_item(id, item, self.data_block_area);
            } else {
                let text = format!("Cannot find data for '{id}'");
                self.popup(text, PopupLevel::Error);
            }
        } else {
            self.data_block.reset();
        }

        if let Some(header) = self.header.as_ref() {
            if !self.skip_header {
                header.draw(frame, self.header_area);
            }
        }

        if let Some(footer) = self.footer.as_ref() {
            let text = match self.foot_message.take() {
                Some(message) => FooterText::Message(message),
                None => {
                    let roots = self.tree_overview.get_root_identifies();
                    let identify = self.tree_overview.get_selected();
                    if roots.is_empty() && identify.is_none() {
                        FooterText::None
                    } else {
                        FooterText::Identify(roots, identify)
                    }
                }
            };

            if !self.skip_footer {
                footer.draw(frame, self.footer_area, text);
            }
        }

        if let ElementInFocus::Popup = self.focus {
            if self.popup.is_disabled() {
                self.disable_popup();
            }
        }

        if let Some(filter) = self.filter.as_mut() {
            let filter_focus = matches!(self.focus, ElementInFocus::Filter);
            filter.draw(frame, self.filter_area, filter_focus);
        }

        let tree_focus = matches!(self.focus, ElementInFocus::TreeOverview);
        self.tree_overview
            .draw(frame, self.tree_overview_area, tree_focus);

        let data_focus = matches!(self.focus, ElementInFocus::DataBlock);
        self.data_block
            .draw(frame, self.data_block_area, data_focus);

        if matches!(self.focus, ElementInFocus::Popup) {
            self.popup.draw(frame);
        }
    }

    fn popup(&mut self, text: impl ToString, level: PopupLevel) {
        self.popup.set_data(text.to_string(), level);
        if !matches!(self.focus, ElementInFocus::Popup | ElementInFocus::None) {
            self.before_popup_focus = self.focus;
        }
        self.focus = ElementInFocus::Popup;
    }

    fn disable_popup(&mut self) {
        if !matches!(
            self.before_popup_focus,
            ElementInFocus::Popup | ElementInFocus::None
        ) {
            self.focus = self.before_popup_focus;
            return;
        }

        self.focus = ElementInFocus::TreeOverview;
    }

    fn refresh_area(&mut self, frame: &Frame) {
        let tree_size = self.layout_tree_size;
        let data_size = 100_u16.saturating_sub(tree_size);

        // These checks should be done in config validation.
        debug_assert_ne!(tree_size, 0);
        debug_assert_ne!(data_size, 0);

        let frame_area = frame.area();
        let main_area = match self.header {
            Some(_) => {
                let Rect { height, .. } = frame_area;
                if height <= Self::HEADER_HEIGHT + 1 {
                    // God knows under what circumstances such a small terminal would appear!
                    // We will not render the header.
                    self.skip_header = true;
                    frame_area
                } else {
                    self.skip_header = false;
                    self.header_area = Rect {
                        height: Self::HEADER_HEIGHT,
                        y: 0,
                        ..frame_area
                    };
                    Rect {
                        height: height.saturating_sub(Self::HEADER_HEIGHT),
                        y: Self::HEADER_HEIGHT,
                        ..frame_area
                    }
                }
            }
            None => frame_area,
        };

        let main_area = match self.footer {
            Some(_) => {
                let Rect { height, .. } = main_area;
                if height <= Self::FOOTER_HEIGHT + 1 {
                    self.skip_footer = true;
                    main_area
                } else {
                    self.skip_footer = false;
                    self.footer_area = Rect {
                        height: Self::FOOTER_HEIGHT,
                        y: height,
                        ..main_area
                    };
                    Rect {
                        height: height.saturating_sub(Self::FOOTER_HEIGHT),
                        ..main_area
                    }
                }
            }
            None => main_area,
        };

        match self.layout_direction {
            LayoutDirection::Vertical => {
                let vertical = Layout::vertical([
                    Constraint::Percentage(tree_size),
                    Constraint::Percentage(data_size),
                ]);
                [self.tree_overview_area, self.data_block_area] = vertical.areas(main_area);
            }
            LayoutDirection::Horizontal => {
                let horizontal = Layout::horizontal([
                    Constraint::Percentage(tree_size),
                    Constraint::Percentage(data_size),
                ]);
                [self.tree_overview_area, self.data_block_area] = horizontal.areas(main_area);
            }
        }
        if self.filter.is_some() {
            // Show filter input text area
            let vertical =
                Layout::vertical([Constraint::Length(Self::FILTER_HEIGHT), Constraint::Min(0)]);
            [self.filter_area, self.tree_overview_area] = vertical.areas(self.tree_overview_area);
        }
    }

    fn can_switch_to_data_block(&self) -> bool {
        match self.focus {
            ElementInFocus::TreeOverview => self.tree_overview.get_selected().is_some(),
            ElementInFocus::None => true,
            ElementInFocus::Filter => true,
            ElementInFocus::DataBlock => false,
            ElementInFocus::Popup => false,
        }
    }

    fn on_key(&mut self, key: KeyEvent) -> Refresh {
        let ka = match self.cfg.keys.get_key_action(key) {
            Some(ka) => ka,
            None => return Refresh::Skip,
        };

        if matches!(self.focus, ElementInFocus::Filter) {
            if let Some(filter) = self.filter.as_mut() {
                let filter_action = filter.on_key(ka);
                match filter_action {
                    FilterAction::Edit => {
                        return Refresh::Update;
                    }
                    FilterAction::Confirm => {
                        self.focus = ElementInFocus::TreeOverview;
                        return Refresh::Update;
                    }
                    FilterAction::Quit => {
                        self.filter = None;
                        self.focus = ElementInFocus::TreeOverview;
                        return Refresh::Update;
                    }
                    FilterAction::Skip => {}
                };
            }
        }

        let action = match ka.action {
            Some(action) => action,
            None => return Refresh::Skip,
        };

        match action {
            Action::Quit => Refresh::Quit,
            Action::Switch => match self.focus {
                ElementInFocus::TreeOverview if self.can_switch_to_data_block() => {
                    self.focus = ElementInFocus::DataBlock;
                    Refresh::Update
                }
                ElementInFocus::DataBlock => {
                    self.focus = ElementInFocus::TreeOverview;
                    Refresh::Update
                }
                _ => Refresh::Skip,
            },
            Action::ChangeLayout => {
                match self.layout_direction {
                    LayoutDirection::Vertical => {
                        self.layout_direction = LayoutDirection::Horizontal
                    }
                    LayoutDirection::Horizontal => {
                        self.layout_direction = LayoutDirection::Vertical
                    }
                }
                Refresh::Update
            }
            Action::TreeScaleUp => {
                if self.layout_tree_size == Config::MAX_LAYOUT_TREE_SIZE {
                    return Refresh::Skip;
                }

                self.layout_tree_size += 2;
                if self.layout_tree_size > Config::MAX_LAYOUT_TREE_SIZE {
                    self.layout_tree_size = Config::MAX_LAYOUT_TREE_SIZE;
                }

                Refresh::Update
            }
            Action::TreeScaleDown => {
                if self.layout_tree_size == Config::MIN_LAYOUT_TREE_SIZE {
                    return Refresh::Skip;
                }

                self.layout_tree_size = self.layout_tree_size.saturating_sub(2);
                if self.layout_tree_size < Config::MIN_LAYOUT_TREE_SIZE {
                    self.layout_tree_size = Config::MIN_LAYOUT_TREE_SIZE;
                }

                Refresh::Update
            }
            Action::Edit => {
                if !matches!(
                    self.focus,
                    ElementInFocus::DataBlock | ElementInFocus::TreeOverview
                ) {
                    return Refresh::Skip;
                }

                let edit = match self.build_edit() {
                    Some(edit) => edit,
                    None => return Refresh::Skip,
                };
                Refresh::Edit(Box::new(edit))
            }
            Action::CopyName | Action::CopyValue => {
                let text = match self.get_copy_text(action) {
                    Some(text) => text,
                    None => return Refresh::Skip,
                };

                if let Err(err) = write_clipboard(&text) {
                    let message = format!("Failed to copy text to clipboard: {err:#}");
                    self.popup(message, PopupLevel::Error);
                    return Refresh::Update;
                }

                let size = humansize::format_size(text.len(), humansize::BINARY);
                let copy_message = format!("copied {size} data to system clipboard");
                self.foot_message = Some(copy_message);
                Refresh::Update
            }
            Action::Filter => {
                if self.cfg.filter.disable {
                    // Filter is disabled, do not handle the action
                    return Refresh::Skip;
                }

                if self.filter.is_none() {
                    self.filter = Some(Filter::new(self.cfg.clone()));
                }
                self.focus = ElementInFocus::Filter;

                Refresh::Update
            }
            _ => {
                // These actions are handled by the focused widget
                if match self.focus {
                    ElementInFocus::TreeOverview => {
                        if self.filter.is_some() && matches!(action, Action::Reset) {
                            self.filter = None;
                            return Refresh::Update;
                        }
                        self.tree_overview.on_key(action)
                    }
                    ElementInFocus::DataBlock => self.data_block.on_key(action),
                    ElementInFocus::Popup => self.popup.on_key(action),
                    ElementInFocus::None | ElementInFocus::Filter => false,
                } {
                    Refresh::Update
                } else {
                    Refresh::Skip
                }
            }
        }
    }

    fn on_click(&mut self, column: u16, row: u16) -> Refresh {
        if matches!(self.focus, ElementInFocus::Popup) {
            self.popup.disable();
            return Refresh::Update;
        }

        if Self::get_row_inside(column, row, self.tree_overview_area).is_some() {
            self.tree_overview.on_click(column, row);
            self.focus = ElementInFocus::TreeOverview;
            return Refresh::Update;
        }

        if Self::get_row_inside(column, row, self.data_block_area).is_some() {
            return if self.can_switch_to_data_block() {
                self.focus = ElementInFocus::DataBlock;
                Refresh::Update
            } else {
                Refresh::Skip
            };
        }

        Refresh::Skip
    }

    fn on_scroll(&mut self, direction: ScrollDirection, column: u16, row: u16) -> Refresh {
        if matches!(self.focus, ElementInFocus::Popup) {
            if self.popup.on_scroll(direction) {
                return Refresh::Update;
            }

            return Refresh::Skip;
        }

        let update = if Self::get_row_inside(column, row, self.tree_overview_area).is_some() {
            self.tree_overview.on_scroll(direction)
        } else if Self::get_row_inside(column, row, self.data_block_area).is_some() {
            self.data_block.on_scroll(direction)
        } else {
            false
        };

        if update {
            Refresh::Update
        } else {
            Refresh::Skip
        }
    }

    fn on_focus_changed(&mut self, focus: bool) -> Refresh {
        if focus {
            if !matches!(self.focus, ElementInFocus::None) {
                // We are already focused, no need to update
                return Refresh::Skip;
            }

            // This implements the functionality to refocus on the last focused widget when
            // we return after losing focus.
            let last_focus = self
                .last_focus
                .take()
                .unwrap_or(ElementInFocus::TreeOverview);
            self.focus = last_focus;
            return Refresh::Update;
        }

        if matches!(self.focus, ElementInFocus::None) {
            // We are already not focused, no need to update
            return Refresh::Skip;
        }

        self.last_focus = Some(self.focus);
        self.focus = ElementInFocus::None;

        Refresh::Update
    }

    fn get_row_inside(column: u16, row: u16, area: Rect) -> Option<u16> {
        if area.contains(Position { x: column, y: row }) {
            Some(row.saturating_sub(area.top()).saturating_sub(1))
        } else {
            None
        }
    }

    fn build_edit(&self) -> Option<Edit> {
        let identify = self.tree_overview.get_selected()?;
        let item = self.tree_overview.get_value(identify.as_str())?;

        let simple_value = match &item.value {
            Value::String(s) => Some(s.clone()),
            Value::Null => Some(String::from("null")),
            Value::Number(num) => Some(num.to_string()),
            Value::Bool(b) => Some(b.to_string()),
            _ => None,
        };

        if let Some(simple_value) = simple_value {
            return Some(Edit::new(self.cfg.as_ref(), identify, simple_value, "txt"));
        }

        let parser = self.tree_overview.get_parser();
        let data = parser.to_string(&item.value);
        let extension = parser.extension();
        Some(Edit::new(self.cfg.as_ref(), identify, data, extension))
    }

    fn get_copy_text(&self, action: Action) -> Option<String> {
        let identify = self.tree_overview.get_selected()?;
        let item = self.tree_overview.get_value(identify.as_str())?;

        if matches!(action, Action::CopyName) {
            return Some(item.name.clone());
        }

        let data = match &item.value {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            _ => {
                let parser = self.tree_overview.get_parser();
                parser.to_string(&item.value)
            }
        };

        Some(data)
    }
}

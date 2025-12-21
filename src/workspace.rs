use std::slice::Iter;
use xcb::x::Window;

#[derive(Default, Debug)]
pub struct Workspace {
    windows: Vec<Window>,
    focus: Option<usize>,
}

impl Workspace {
    pub fn get_focused_window(&self) -> Option<&Window> {
        self.focus.and_then(|i| self.windows.get(i))
    }

    pub fn num_of_windows(&self) -> usize {
        self.windows.len()
    }

    pub fn set_focus(&mut self, idx: usize) -> bool {
        if idx >= self.windows.len() {
            return false;
        }
        self.focus = Some(idx);
        true
    }

    pub fn get_focus(&self) -> Option<usize> {
        self.focus
    }

    pub fn push_window(&mut self, window: Window) {
        self.windows.push(window);
        if self.focus.is_none() {
            self.focus = Some(self.windows.len().saturating_sub(1));
        }
    }

    pub fn remove_window(&mut self, idx: usize) -> Option<Window> {
        if idx < self.num_of_windows() {
            let window = self.windows.remove(idx);
            self.update_focus();
            return Some(window);
        }
        None
    }

    fn update_focus(&mut self) {
        if self.windows.is_empty() {
            self.focus = None;
            return;
        }

        match self.focus {
            Some(f) if f < self.windows.len() => {}
            _ => self.focus = Some(self.windows.len().saturating_sub(1)),
        }
    }

    pub fn removed_focused_window(&mut self) -> Option<Window> {
        if let Some(idx) = self.focus {
            self.remove_window(idx)
        } else {
            None
        }
    }

    pub fn iter_windows(&self) -> Iter<'_, Window> {
        self.windows.iter()
    }

    pub fn retain<F: FnMut(&Window) -> bool>(&mut self, f: F) {
        self.windows.retain(f)
    }
}

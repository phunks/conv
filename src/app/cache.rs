use std::sync::Arc;
use std::time::Instant;

use difftastic::clipboard::ClipboardDiff;
use eframe::egui::{text, Galley, Style, TextFormat, TextStyle, Ui};

use crate::app::result::ConvertResult;
use crate::app::state::SelectedTool;
use crate::widgets::input_editor::LayoutCache;

#[derive(Default)]
pub struct AppCache {
    pub outputs: OutputCaches,
    pub input_layout: LayoutCache,
    pub output_layout: TextGalleyCache,
    pub diff: DiffCache,
}

#[derive(Default)]
pub struct DiffCache {
    pub result: Option<ClipboardDiff>,
    pub error: Option<String>,
    pub update_deadline: Option<Instant>,
}

#[derive(Default)]
pub struct TextGalleyCache {
    #[allow(unused)]
    style: Style,
    text: String,
    #[allow(unused)]
    wrap_width: f32,
    galley: Option<Arc<Galley>>,
}

impl TextGalleyCache {
    #[allow(unused)]
    pub fn galley(
        &mut self,
        ui: &Ui,
        text: &str,
        wrap_width: f32,
    ) -> Arc<Galley> {
        if self.galley.is_none()
            || self.text != text
            || self.style != *ui.style().as_ref()
            || (self.wrap_width - wrap_width).abs() > f32::EPSILON
        {
            self.style = ui.style().as_ref().clone();
            text.clone_into(&mut self.text);
            self.wrap_width = wrap_width;

            let mut job = text::LayoutJob::default();

            if !text.is_empty() {
                job.append(
                    text,
                    0.0,
                    TextFormat {
                        font_id: TextStyle::Body.resolve(ui.style()),
                        color: ui.visuals().text_color(),
                        ..Default::default()
                    },
                );
            }

            job.wrap.max_width = wrap_width;

            self.galley = Some(ui.fonts(|fonts| fonts.layout_job(job)));
        }

        self.galley
            .as_ref()
            .expect("galley cache must be initialized")
            .clone()
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.galley = None;
    }
}

pub struct OutputCaches {
    items: [OutputCache; SelectedTool::COUNT],
}

impl OutputCaches {
    pub fn current(&self, selected: SelectedTool) -> &OutputCache {
        &self.items[selected.index()]
    }

    pub fn current_mut(&mut self, selected: SelectedTool) -> &mut OutputCache {
        &mut self.items[selected.index()]
    }

    pub fn mark_current_dirty(&mut self, selected: SelectedTool) {
        self.current_mut(selected).dirty = true;
    }

    pub fn mark_all_dirty(&mut self) {
        for item in &mut self.items {
            item.dirty = true;
        }
    }
}

impl Default for OutputCaches {
    fn default() -> Self {
        Self {
            items: std::array::from_fn(|_| OutputCache::default()),
        }
    }
}

pub struct OutputCache {
    pub dirty: bool,
    pub result: ConvertResult,
}

impl Default for OutputCache {
    fn default() -> Self {
        Self {
            dirty: true,
            result: ConvertResult::default(),
        }
    }
}
use std::sync::Arc;
use std::time::Instant;

use difftastic::clipboard::ClipboardDiff;
use eframe::egui::{text, Galley, Style, TextFormat, TextStyle, Ui};

use crate::app::result::ConvertResult;
use crate::app::state::SelectedTool;
use crate::widgets::input_editor::LayoutCache;

// Display-only marker indicating a virtual empty line.
// Exclude in canonical source and clipboard output.
pub const VIRTUAL_LINE_MARKER: char = '\u{25B8}';

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
    pub aligned: Option<AlignedDiff>,
    pub error: Option<String>,
    pub update_deadline: Option<Instant>,
    pub hex: DiffHexCache,
    pub layout: DiffLayoutCache,
    pub selection: DiffSelectionCache,
}

#[derive(Default)]
pub struct DiffSelectionCache {
    pub left: Option<std::ops::Range<usize>>,
    pub right: Option<std::ops::Range<usize>>,
}

#[derive(Default)]
pub struct DiffLayoutCache {
    pub left: DiffPaneGalleyCache,
    pub right: DiffPaneGalleyCache,
}

#[derive(Default)]
pub struct DiffPaneGalleyCache {
    style: Style,
    text: String,
    wrap_width: f32,
    galley: Option<Arc<Galley>>,
}

impl DiffPaneGalleyCache {
    pub fn galley(
        &mut self,
        ui: &Ui,
        text: &str,
        wrap_width: f32,
        create_job: impl FnOnce() -> text::LayoutJob,
    ) -> Arc<Galley> {
        let needs_update = self.galley.is_none()
            || self.text != text
            || self.style != *ui.style().as_ref()
            || (self.wrap_width - wrap_width).abs() > f32::EPSILON;

        if needs_update {
            self.style = ui.style().as_ref().clone();
            text.clone_into(&mut self.text);
            self.wrap_width = wrap_width;

            self.galley = Some(ui.fonts(|fonts| fonts.layout_job(create_job())));
        }

        self.galley
            .as_ref()
            .expect("diff galley cache must be initialized")
            .clone()
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.galley = None;
    }
}

#[derive(Default)]
pub struct DiffHexCache {
    pub left: HexLineCache,
    pub right: HexLineCache,
}

#[derive(Default)]
pub struct HexLineCache {
    pub line_number: Option<usize>,
    pub source_line: String,
    pub text: String,
    pub byte_ranges: Vec<std::ops::Range<usize>>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ChangeMarker {
    // The starting display line of the change block.
    pub display_row: usize,

    // End display line (exclusive) of the changed block.
    pub end_display_row: usize,

    // The character offset of the beginning of the changed line
    //  in the aligned text in the left pane.
    pub left_char_offset: usize,

    // Actual scroll destination Y coordinate taken from the last galley.
    pub scroll_y: f32,
}

#[derive(Default)]
pub struct AlignedDiff {
    pub left: AlignedPane,
    pub right: AlignedPane,
    pub left_wrap_width: f32,
    pub right_wrap_width: f32,
    // Actual scroll destination for each changed line.
    pub change_markers: Vec<ChangeMarker>,
    // Total number of displayed lines on both sides,
    // including virtual lines and wrapping.
    pub total_display_rows: usize,
}

#[derive(Default)]
pub struct AlignedPane {
    pub text: String,
    pub line_numbers: String,
    pub lines: Vec<AlignedLine>,
    pub had_trailing_newline: bool,
}

impl AlignedLine {
    pub const fn source(source_line: usize) -> Self {
        Self {
            source_line: Some(source_line),
        }
    }

    pub const fn virtual_blank() -> Self {
        Self { source_line: None }
    }

    pub const fn is_virtual_blank(self) -> bool {
        self.source_line.is_none()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AlignedLine {
    pub source_line: Option<usize>,
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
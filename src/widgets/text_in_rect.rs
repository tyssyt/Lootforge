use crate::prelude::*;

pub fn text_in_rect(
    ui: &mut Ui,
    text: impl Into<RichText>,
    rect: Rect,
    align: Align2,
) {
    let layout_job = WidgetText::RichText(text.into()).into_layout_job(
        ui.style(),
        FontSelection::Default,
        align.y(),
    );

    let galley = ui.painter().layout_job(layout_job);
    let pos = align.align_size_within_rect(galley.size(), rect).left_top();

    ui.painter().galley(pos, galley, ui.visuals().text_color());
}
use crate::prelude::*;

pub fn text_in_rect(
    ui: &mut Ui,
    text: impl Into<RichText>,
    color: Color32,
    rect: Rect,
    align: Align2,
) {
    let mut layout_job = WidgetText::RichText(text.into()).into_layout_job(
        ui.style(),
        FontSelection::Default,
        align.y(),
    );
    layout_job.halign = align.x();

    let galley = ui.fonts(|fonts| fonts.layout_job(layout_job));

    let pos = align.pos_in_rect(&rect);

    ui.painter().galley(pos, galley, color);
}
use crate::prelude::*;

use crate::{item::Item, stash::stash::Stash};

// copied & modified from egui::Ui
pub fn item_drop_zone<R>(
    ui: &mut Ui,
    stash: &Stash,
    accepts: impl FnOnce(&Item) -> bool,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> (InnerResponse<R>, Option<Rc<Item>>) {
    let is_anything_being_dragged = DragAndDrop::has_any_payload(ui.ctx());

    let dnd_payload = DragAndDrop::payload::<usize>(ui.ctx())
        .and_then(|id| stash.find(*id))
        .filter(|id| accepts(id.as_ref()));

    let mut frame = Frame::new().inner_margin(5).begin(ui);
    let inner = add_contents(&mut frame.content_ui);
    let response = frame.allocate_space(ui);

    // NOTE: we use `response.contains_pointer` here instead of `hovered`, because
    // `hovered` is always false when another widget is being dragged.
    let style = if is_anything_being_dragged && dnd_payload.is_some() {
        ui.visuals().widgets.active
    } else {
        ui.visuals().widgets.inactive
    };

    let mut fill = style.bg_fill;
    let mut stroke = style.bg_stroke;

    if is_anything_being_dragged && dnd_payload.is_none() {
        // When dragging something else, show that it can't be dropped here:
        fill = ui.visuals().gray_out(fill);
        stroke.color = ui.visuals().gray_out(stroke.color);
    }

    frame.frame.fill = fill;
    frame.frame.stroke = stroke;

    frame.paint(ui);

    let payload = response
        .dnd_release_payload::<usize>()
        .and_then(|_| dnd_payload);

    (InnerResponse::new(inner, response), payload)
}

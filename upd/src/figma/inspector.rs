use super::icon::Icon;
use super::ui::{self, BtnIcon, FILL, Flow, Ui};
use bevy_egui::egui;

const PADDING: egui::Margin = egui::Margin {
    left: 16,
    right: 8,
    top: 4,
    bottom: 4,
};

pub fn panel(mut ui: Ui, state: &mut super::State) {
    {
        ui.ui.add_space(80.0);
        ui.separator();
    }

    {
        let line = Flow::symmetric(8, 8, 4, 40.0, [54.0, FILL, 24.0, 24.0, 24.0]);
        ui.line(line, |[mut label, _, mut a, mut b, mut c]| {
            // > 16x16
            label.btn_header(true, false, "Frame");
            a.btn_center(false, true, "a");
            b.btn_center(false, true, "b");
            c.btn_center(false, true, "c");
        });
        ui.separator();
    }

    edit_position(&mut ui, state);
    edit_layout(&mut ui, state);
    edit_apperance(&mut ui, state);

    ui.header([FILL], |[mut ui]| {
        ui.btn_header(true, false, "Fill");
    });
    ui.separator();
    ui.header([FILL], |[mut ui]| {
        ui.btn_header(true, false, "Stroke");
    });
    ui.separator();
    ui.header([FILL], |[mut ui]| {
        ui.btn_header(true, false, "Layout Guide");
    });
    ui.separator();

    let line = Flow::symmetric(8, 4, 0, 32.0, [FILL]);

    ui.line(line, |[mut ui]| {
        let mut radians = 0.7;
        ui.ui.add(egui::DragValue::new(&mut radians));
    });

    ui.line(line, |[mut ui]| {
        let ui = &mut ui.ui;

        let mut s = String::from("1234");
        let text = egui::TextEdit::singleline(&mut s)
            .horizontal_align(ui.layout().horizontal_align())
            .vertical_align(ui.layout().vertical_align())
            .frame(false);
        ui.add(text);
    });
}

fn edit_position(ui: &mut Ui, state: &mut super::State) {
    use Icon::*;

    ui.header([FILL, 24.0], |[mut ui, mut icon]| {
        ui.btn_header(true, false, "Position");
        icon.btn_icon(Icon::IgnoreAutoLayout, BtnIcon::FRAMELESS);
    });

    let inner = Flow::symmetric(0, 0, 1, 24.0, [FILL, FILL, FILL]);
    let line = Flow::new(PADDING, 8, 32.0, [FILL, FILL, 24.0]);
    line.show(&mut ui.ui, |[mut x, mut y, _]| {
        inner.show(&mut x.ui, |mut ui| {
            ui[0].btn_icon(AlignL, BtnIcon::LEFT);
            ui[1].btn_icon(AlignH, BtnIcon::CENTER);
            ui[2].btn_icon(AlignR, BtnIcon::RIGHT);
        });

        inner.show(&mut y.ui, |mut ui| {
            ui[0].btn_icon(AlignT, BtnIcon::LEFT);
            ui[1].btn_icon(AlignV, BtnIcon::CENTER);
            ui[2].btn_icon(AlignB, BtnIcon::RIGHT);
        });
    });

    let line = Flow::new(PADDING, 8, 32.0, [FILL, FILL, 24.0]);
    line.show(&mut ui.ui, |[mut x, mut y, _icon]| {
        x.edit(X, &mut state.pos.x);
        y.edit(Y, &mut state.pos.y);
    });
    line.show(&mut ui.ui, |[mut x, mut y, _icon]| {
        x.edit(Angle, &mut state.rotation);
        inner.show(&mut y.ui, |mut ui| {
            ui[0].btn_icon(AlignT, BtnIcon::LEFT);
            ui[1].btn_icon(AlignV, BtnIcon::CENTER);
            ui[2].btn_icon(AlignB, BtnIcon::RIGHT);
        });
    });
    ui.ui.add_space(8.0);
    ui.separator();
}

fn edit_layout(ui: &mut Ui, state: &mut super::State) {
    ui.header([FILL, 24.0, 24.0], |[mut title, mut rtf, mut lay]| {
        title.btn_header(true, false, "Layout");
        rtf.btn_icon(Icon::ResizeToFit, BtnIcon::FRAMELESS);
        lay.btn_icon(Icon::EnableAutoLayout, BtnIcon::FRAMELESS);
    });

    let line = Flow::new(PADDING, 8, 32.0, [FILL, FILL, 24.0]);

    line.show(&mut ui.ui, |[mut w, mut h, _icon]| {
        w.edit(Icon::W, &mut state.size.x);
        h.edit(Icon::H, &mut state.size.y);
    });

    line.show(&mut ui.ui, |[mut w, mut h, _icon]| {
        w.edit(Icon::MinW, &mut state.min_size.x);
        h.edit(Icon::MinH, &mut state.min_size.y);
    });
    line.show(&mut ui.ui, |[mut w, mut h, _icon]| {
        w.edit(Icon::MaxW, &mut state.max_size.x);
        h.edit(Icon::MaxH, &mut state.max_size.y);
    });

    line.show(&mut ui.ui, |[mut h, mut v, _icon]| {
        h.edit(Icon::HGap, &mut state.gap.x);
        v.edit(Icon::VGap, &mut state.gap.y);
    });

    line.show(&mut ui.ui, |[mut l, mut t, mut icon]| {
        l.edit(Icon::PadL, &mut state.padding.l);
        t.edit(Icon::PadT, &mut state.padding.t);
        icon.btn_icon(Icon::PadIndividual, BtnIcon::FRAMELESS);
    });
    line.show(&mut ui.ui, |[mut r, mut b, _icon]| {
        r.edit(Icon::PadR, &mut state.padding.r);
        b.edit(Icon::PadB, &mut state.padding.b);
    });

    ui.ui.add_space(8.0);
    ui.separator();
}

fn edit_apperance(ui: &mut Ui, state: &mut super::State) {
    ui.header([FILL, 24.0], |[mut title, mut btn]| {
        title.btn_header(true, false, "Apperance");
        //btn.btn_icon(Icon::Radius, BtnIcon::FRAMELESS);
    });

    let line = Flow::new(PADDING, 8, 32.0, [FILL, FILL, 24.0]);
    line.show(&mut ui.ui, |[mut a, mut b, mut icon]| {
        a.btn_left(true, true, "X");
        b.edit(Icon::Radius, &mut state.radius.ne);
        icon.btn_icon(Icon::Radius, BtnIcon::FRAMELESS);
    });

    line.show(&mut ui.ui, |[mut nw, mut ne, _icon]| {
        nw.edit(Icon::RadiusNW, &mut state.radius.nw);
        ne.edit(Icon::RadiusNE, &mut state.radius.ne);
    });
    line.show(&mut ui.ui, |[mut sw, mut se, _icon]| {
        sw.edit(Icon::RadiusSW, &mut state.radius.sw);
        se.edit(Icon::RadiusSE, &mut state.radius.se);
    });

    ui.ui.add_space(8.0);
    ui.separator();
}

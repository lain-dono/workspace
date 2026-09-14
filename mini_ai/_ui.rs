#[derive(Clone)]
struct NeedData {
    name: String,
    value: f32,
    // curve: CurveFloat<8>,
    color: usize,
}

impl NeedData {
    fn new(name: impl Into<String>, color: usize) -> Self {
        Self {
            name: name.into(),
            value: 20.0,
            // curve: crate::_sims::CurveFloat([1.0, 0.9, 0.75, 0.8, 0.6, 0.5, 0.4, 0.25]),
            // curve: CurveFloat(std::array::from_fn(|i| {
            //     if i == 0 {
            //         1.0
            //     } else if i == 7 {
            //         0.0
            //     } else {
            //         let mut rng = rand::rng();
            //         rng.random_range(0.0..=(1.0 - i as f32 / 16.0))
            //     }
            // })),
            color,
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        let curve = &mut self.curve;

        ui.add(egui::Slider::new(&mut self.value, 0.0..=100.0).text(&self.name));

        ui.columns_const::<8, _>(|ui| {
            for (value, ui) in curve.0.iter_mut().zip(ui) {
                ui.add(egui::DragValue::new(value).range(0.0..=1.0).speed(0.01));
            }
        });
    }

    fn plot(&mut self, ui: &mut egui::Ui) {
        Self::plotter(ui, |ui| self.plot_inner(ui));
    }

    fn plotter(ui: &mut egui::Ui, inner: impl FnOnce(&mut egui_plot::PlotUi)) {
        // let step_size = (keys[1] - keys[0]) as f64;
        let step_size = (1 / (8 - 1)) as f64;
        let grid = |_input| {
            crate::curve::keys::<8>()
                .map(|v| v as f64)
                .map(|value| egui_plot::GridMark { value, step_size })
                .to_vec()
        };

        egui_plot::Plot::new("plot_t")
            .allow_zoom(false)
            .allow_drag(false)
            .allow_scroll(false)
            .show_axes([false; 2])
            .default_x_bounds(-0.0, 1.0)
            .default_y_bounds(-0.0, 1.0)
            // .width(256.0)
            // .height(128.0)
            .height(256.0)
            .x_grid_spacer(grid)
            .legend(Legend::default().follow_insertion_order(true))
            .show(ui, inner);
    }

    fn plot_inner(&mut self, plot_ui: &mut egui_plot::PlotUi) {
        let color = acolor(self.color);

        let curve = self.curve;
        let curve2 = CurveUnorm8::<8>::from_map(|input| curve.linear(input));

        let scorer = move |x: f64| mono_curve_unorm8(x as f32, curve2.0) as f64;
        let points = PlotPoints::from_explicit_callback(scorer, -2.0..2.0, 5000);
        plot_ui.line(Line::new(&self.name, points).width(0.5).color(color));

        let t = self.value as f64 / 100.0;
        let series = vec![[t, scorer(t)]];
        plot_ui.points(Points::new(&self.name, series).radius(2.5).color(color));
    }
}

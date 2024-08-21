

pub struct BarChart {
    pub bars: Vec<Bar>,
    pub default_color: Color32,
    pub name: String,

    pub element_formatter: Option<Box<dyn Fn(&Bar, &BarChart) -> String>>,

    highlight: bool,
    allow_hover: bool,
    id: Option<Id>,
}
impl BarChart {
    /// Create a bar chart. It defaults to vertically oriented elements.
    pub fn new(bars: Vec<Bar>) -> Self {
        Self {
            bars,
            default_color: Color32::TRANSPARENT,
            name: String::new(),
            element_formatter: None,
            highlight: false,
            allow_hover: true,
            id: None,
        }
    }

        /// Set the default color. It is set on all elements that do not already have a specific color.
    /// This is the color that shows up in the legend.
    /// It can be overridden at the bar level (see [[`Bar`]]).
    /// Default is `Color32::TRANSPARENT` which means a color will be auto-assigned.
    #[inline]
    pub fn color(mut self, color: impl Into<Color32>) -> Self {
        let plot_color = color.into();
        self.default_color = plot_color;
        for b in &mut self.bars {
            if b.fill == Color32::TRANSPARENT && b.stroke.color == Color32::TRANSPARENT {
                b.fill = plot_color.linear_multiply(0.2);
                b.stroke.color = plot_color;
            }
        }
        self
    }


    /// Name of this chart.
    ///
    /// This name will show up in the plot legend, if legends are turned on. Multiple charts may
    /// share the same name, in which case they will also share an entry in the legend.
    #[allow(clippy::needless_pass_by_value)]
    #[inline]
    pub fn name(mut self, name: impl ToString) -> Self {
        self.name = name.to_string();
        self
    }
        /// Set all elements to be in a vertical orientation.
    /// Argument axis will be X and bar values will be on the Y axis.
    #[inline]
    pub fn vertical(mut self) -> Self {
        for b in &mut self.bars {
            b.orientation = Orientation::Vertical;
        }
        self
    }


    /// Set all elements to be in a horizontal orientation.
    /// Argument axis will be Y and bar values will be on the X axis.
    #[inline]
    pub fn horizontal(mut self) -> Self {
        for b in &mut self.bars {
            b.orientation = Orientation::Horizontal;
        }
        self
    }

    /// Set the width (thickness) of all its elements.
    #[inline]
    pub fn width(mut self, width: f64) -> Self {
        for b in &mut self.bars {
            b.bar_width = width;
        }
        self
    }

    /// Highlight all plot elements.
    #[inline]
    pub fn highlight(mut self, highlight: bool) -> Self {
        self.highlight = highlight;
        self
    }

    /// Allowed hovering this item in the plot. Default: `true`.
    #[inline]
    pub fn allow_hover(mut self, hovering: bool) -> Self {
        self.allow_hover = hovering;
        self
    }

    /// Add a custom way to format an element.
    /// Can be used to display a set number of decimals or custom labels.
    #[inline]
    pub fn element_formatter(mut self, formatter: Box<dyn Fn(&Bar, &Self) -> String>) -> Self {
        self.element_formatter = Some(formatter);
        self
    }

    /// Stacks the bars on top of another chart.
    /// Positive values are stacked on top of other positive values.
    /// Negative values are stacked below other negative values.
    #[inline]
    pub fn stack_on(mut self, others: &[&Self]) -> Self {
        for (index, bar) in self.bars.iter_mut().enumerate() {
            let new_base_offset = if bar.value.is_sign_positive() {
                others
                    .iter()
                    .filter_map(|other_chart| other_chart.bars.get(index).map(|bar| bar.upper()))
                    .max_by_key(|value| value.ord())
            } else {
                others
                    .iter()
                    .filter_map(|other_chart| other_chart.bars.get(index).map(|bar| bar.lower()))
                    .min_by_key(|value| value.ord())
            };

            if let Some(value) = new_base_offset {
                bar.base_offset = Some(value);
            }
        }
        self
    }

    /// Set the bar chart's id which is used to identify it in the plot's response.
    #[inline]
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }
}

impl PlotItem for BarChart {
    fn shapes(&self, _ui: &Ui, transform: &PlotTransform, shapes: &mut Vec<Shape>) {
        for b in &self.bars {
            b.add_shapes(transform, self.highlight, shapes);
        }
    }

    fn initialize(&mut self, _x_range: RangeInclusive<f64>) {
        // nothing to do
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn color(&self) -> Color32 {
        self.default_color
    }

    fn highlight(&mut self) {
        self.highlight = true;
    }

    fn highlighted(&self) -> bool {
        self.highlight
    }

    fn allow_hover(&self) -> bool {
        self.allow_hover
    }

    fn geometry(&self) -> PlotGeometry<'_> {
        PlotGeometry::Rects
    }

    fn bounds(&self) -> PlotBounds {
        let mut bounds = PlotBounds::NOTHING;
        for b in &self.bars {
            bounds.merge(&b.bounds());
        }
        bounds
    }

    fn find_closest(&self, point: Pos2, transform: &PlotTransform) -> Option<ClosestElem> {
        find_closest_rect(&self.bars, point, transform)
    }

    fn on_hover(
        &self,
        elem: ClosestElem,
        shapes: &mut Vec<Shape>,
        cursors: &mut Vec<Cursor>,
        plot: &PlotConfig<'_>,
        _: &LabelFormatter,
    ) {
        let bar = &self.bars[elem.index];

        bar.add_shapes(plot.transform, true, shapes);
        bar.add_rulers_and_text(self, plot, shapes, cursors);
    }

    fn id(&self) -> Option<Id> {
        self.id
    }
}
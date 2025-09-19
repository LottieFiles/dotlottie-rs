#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Fit {
    Contain,
    Fill,
    Cover,
    FitWidth,
    FitHeight,
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Layout {
    pub fit: Fit,
    pub align: Vec<f32>,
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            fit: Fit::Contain,
            align: vec![0.5, 0.5],
        }
    }
}

impl Layout {
    pub fn new(fit: Fit, align: Vec<f32>) -> Self {
        Self {
            fit,
            align: validate_normalize_align(align),
        }
    }

    pub fn to_transform_matrix(
        &self,
        canvas_width: f32,
        canvas_height: f32,
        paint_width: f32,
        paint_height: f32,
    ) -> [f32; 9] {
        let mut scale_x = 1.0;
        let mut scale_y = 1.0;

        match self.fit {
            // Ensures the entire animation is visible within the canvas while maintaining aspect ratio.
            Fit::Contain => {
                scale_x = canvas_width / paint_width;
                scale_y = canvas_height / paint_height;
                let scale = scale_x.min(scale_y);
                scale_x = scale;
                scale_y = scale;
            }
            // Stretches the animation to fill the entire canvas, potentially distorting aspect ratio
            Fit::Fill => {
                scale_x = canvas_width / paint_width;
                scale_y = canvas_height / paint_height;
            }
            // Enlarges or shrinks the animation to cover the entire canvas.
            Fit::Cover => {
                scale_x = canvas_width / paint_width;
                scale_y = canvas_height / paint_height;
                let scale = scale_x.max(scale_y);
                scale_x = scale;
                scale_y = scale;
            }
            // Scales the animation to fit the canvas width while maintaining aspect ratio.
            Fit::FitWidth => {
                scale_x = canvas_width / paint_width;
                scale_y = scale_x;
            }
            // Scales the animation to fit the canvas height while maintaining aspect ratio.
            Fit::FitHeight => {
                scale_y = canvas_height / paint_height;
                scale_x = scale_y;
            }
            // Disables any scaling, rendering the animation at its original size.
            Fit::None => {}
        };

        let scaled_width = paint_width * scale_x;
        let scaled_height = paint_height * scale_y;

        let shift_x = (canvas_width - scaled_width) * self.align[0];
        let shift_y = (canvas_height - scaled_height) * self.align[1];

        [scale_x, 0.0, shift_x, 0.0, scale_y, shift_y, 0.0, 0.0, 1.0]
    }
}

fn validate_normalize_align(align: Vec<f32>) -> Vec<f32> {
    let mut align = align;

    if align.len() != 2 {
        align = vec![0.5, 0.5];
    }

    align[0] = align[0].clamp(0.0, 1.0);
    align[1] = align[1].clamp(0.0, 1.0);

    align
}

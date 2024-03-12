#[derive(Clone, PartialEq)]
pub enum Fit {
    Contain,
    Fill,
    Cover,
    FitWidth,
    FitHeight,
    None,
}

#[derive(Clone, PartialEq)]
pub struct Layout {
    pub fit: Fit,
    pub align: Vec<f32>,
}

impl Layout {
    pub fn new(fit: Fit, align: Vec<f32>) -> Self {
        Self {
            fit,
            align: validate_normalize_align(align),
        }
    }

    pub fn default() -> Self {
        Self {
            fit: Fit::Contain,
            align: vec![0.5, 0.5],
        }
    }

    pub fn compute_layout_transform(
        &self,
        canvas_width: f32,
        canvas_height: f32,
        picture_width: f32,
        picture_height: f32,
    ) -> (f32, f32, f32, f32) {
        let mut scale_x = 1.0;
        let mut scale_y = 1.0;

        match self.fit {
            // Ensures the entire animation is visible within the canvas while maintaining aspect ratio.
            Fit::Contain => {
                scale_x = canvas_width / picture_width;
                scale_y = canvas_height / picture_height;
                let scale = scale_x.min(scale_y);
                scale_x = scale;
                scale_y = scale;
            }
            // Stretches the animation to fill the entire canvas, potentially distorting aspect ratio
            Fit::Fill => {
                scale_x = canvas_width / picture_width;
                scale_y = canvas_height / picture_height;
            }
            // Enlarges or shrinks the animation to cover the entire canvas.
            Fit::Cover => {
                scale_x = canvas_width / picture_width;
                scale_y = canvas_height / picture_height;
                let scale = scale_x.max(scale_y);
                scale_x = scale;
                scale_y = scale;
            }
            // Scales the animation to fit the canvas width while maintaining aspect ratio.
            Fit::FitWidth => {
                scale_x = canvas_width / picture_width;
                scale_y = scale_x;
            }
            // Scales the animation to fit the canvas height while maintaining aspect ratio.
            Fit::FitHeight => {
                scale_y = canvas_height / picture_height;
                scale_x = scale_y;
            }
            // Disables any scaling, rendering the animation at its original size.
            Fit::None => {}
        };

        let scaled_width = picture_width * scale_x;
        let scaled_height = picture_height * scale_y;

        let shift_x = (canvas_width - scaled_width) * self.align[0];
        let shift_y = (canvas_height - scaled_height) * self.align[1];

        (scaled_width, scaled_height, shift_x, shift_y)
    }
}

fn validate_normalize_align(align: Vec<f32>) -> Vec<f32> {
    let mut align = align;

    if align.len() != 2 {
        align = vec![0.5, 0.5];
    }

    align[0] = align[0].max(0.0).min(1.0);
    align[1] = align[1].max(0.0).min(1.0);

    align
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gen_align(preset: &str) -> Vec<f32> {
        match preset {
            "center-left" => vec![0.0, 0.5],
            "center" => vec![0.5, 0.5],
            "center-right" => vec![1.0, 0.5],

            "top-left" => vec![0.0, 0.0],
            "top-center" => vec![0.5, 0.0],
            "top-right" => vec![1.0, 0.0],

            "bottom-left" => vec![0.0, 1.0],
            "bottom-center" => vec![0.5, 1.0],
            "bottom-right" => vec![1.0, 1.0],

            _ => vec![0.5, 0.5],
        }
    }

    #[test]
    fn layout_tests() {
        let tests = vec![
            (
                "Contain center_left",
                Layout {
                    fit: Fit::Contain,
                    align: gen_align("center-left"),
                },
                (200.0, 200.0, 100.0, 50.0),
                (200.0, 100.0, 0.0, 50.0),
            ),
            (
                "Contain center",
                Layout {
                    fit: Fit::Contain,
                    align: gen_align("center"),
                },
                (200.0, 200.0, 100.0, 50.0),
                (200.0, 100.0, 0.0, 50.0),
            ),
            (
                "Contain center-right",
                Layout {
                    fit: Fit::Contain,
                    align: gen_align("center-right"),
                },
                (200.0, 200.0, 100.0, 50.0),
                (200.0, 100.0, 0.0, 50.0),
            ),
            (
                "Contain top-left",
                Layout {
                    fit: Fit::Contain,
                    align: gen_align("top-left"),
                },
                (200.0, 200.0, 100.0, 50.0),
                (200.0, 100.0, 0.0, 0.0),
            ),
            (
                "Contain top-center",
                Layout {
                    fit: Fit::Contain,
                    align: gen_align("top-center"),
                },
                (200.0, 200.0, 100.0, 50.0),
                (200.0, 100.0, 0.0, 0.0),
            ),
            (
                "Contain top-right",
                Layout {
                    fit: Fit::Contain,
                    align: gen_align("top-right"),
                },
                (200.0, 200.0, 100.0, 50.0),
                (200.0, 100.0, 0.0, 0.0),
            ),
            (
                "Contain bottom-left",
                Layout {
                    fit: Fit::Contain,
                    align: gen_align("bottom-left"),
                },
                (200.0, 200.0, 100.0, 50.0),
                (200.0, 100.0, 0.0, 100.0),
            ),
            (
                "Contain bottom-center",
                Layout {
                    fit: Fit::Contain,
                    align: gen_align("bottom-center"),
                },
                (200.0, 200.0, 100.0, 50.0),
                (200.0, 100.0, 0.0, 100.0),
            ),
            (
                "Contain bottom-right",
                Layout {
                    fit: Fit::Contain,
                    align: gen_align("bottom-right"),
                },
                (200.0, 200.0, 100.0, 50.0),
                (200.0, 100.0, 0.0, 100.0),
            ),
        ];

        for (
            case_name,
            layout,
            (canvas_width, canvas_height, picture_width, picture_height),
            expected,
        ) in tests
        {
            let actual = layout.compute_layout_transform(
                canvas_width,
                canvas_height,
                picture_width,
                picture_height,
            );
            assert!(
                nearly_equal(actual, expected, 0.01),
                "Case '{}': expected {:?}, got {:?}",
                case_name,
                expected,
                actual
            );
        }
    }

    // Helper function to compare float tuples with a tolerance
    fn nearly_equal(a: (f32, f32, f32, f32), b: (f32, f32, f32, f32), epsilon: f32) -> bool {
        (a.0 - b.0).abs() < epsilon
            && (a.1 - b.1).abs() < epsilon
            && (a.2 - b.2).abs() < epsilon
            && (a.3 - b.3).abs() < epsilon
    }
}

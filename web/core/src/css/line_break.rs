use super::{layout::CSSPixels, FontMetrics};

/// Breaks Paragraphs into lines based on their width
pub struct LineBreakIterator<'a> {
    /// The maximum width available for individual line boxes.
    ///
    /// Note that this is just a guideline, line boxes may overflow
    /// if they cannot be broken up.
    available_width: CSSPixels,
    font_metrics: FontMetrics,
    text: &'a str,
    is_finished: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct TextLine<'a> {
    pub text: &'a str,
    pub width: CSSPixels,
}

impl<'a> LineBreakIterator<'a> {
    #[inline]
    #[must_use]
    pub fn new(text: &'a str, font_metrics: FontMetrics, available_width: CSSPixels) -> Self {
        Self {
            text: text.trim_start(),
            font_metrics,
            available_width,
            is_finished: false,
        }
    }
}

impl<'a> Iterator for LineBreakIterator<'a> {
    type Item = TextLine<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_finished {
            return None;
        }

        let mut previous_potential_breakpoint = None;
        let potential_breaks = self
            .text
            .match_indices(char::is_whitespace)
            .map(|(index, _)| index);

        for break_point in potential_breaks {
            let (line, remainder) = self.text.split_at(break_point);

            let width = CSSPixels(
                self.font_metrics
                    .font_face
                    .compute_rendered_width(line, self.font_metrics.size.into()),
            );

            if width <= self.available_width {
                // No need to break yet
                previous_potential_breakpoint = Some((line, remainder, width));
                continue;
            } else {
                // We've exceeded the available space
                match previous_potential_breakpoint {
                    Some((line, remainder, width)) => {
                        // There was a valid potential breakpoint, let's use that one instead
                        self.text = remainder.trim_start();
                        return Some(TextLine { text: line, width });
                    },
                    None => {
                        // Our line is too wide, but there was no opportunity to split it.
                        // Let's just return it as a whole
                        self.text = remainder.trim_start();
                        return Some(TextLine { text: line, width });
                    },
                }
            }
        }

        // There are no further opportunities to split this text
        // Return it as one single line box, ignoring the width that
        // is actually available
        let width = CSSPixels(
            self.font_metrics
                .font_face
                .compute_rendered_width(self.text, self.font_metrics.size.into()),
        );

        self.is_finished = true;

        Some(TextLine {
            text: self.text,
            width,
        })
    }
}

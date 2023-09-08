use super::{layout::CSSPixels, FontMetrics};

/// Breaks Paragraphs into lines based on their width
pub struct LineBreakIterator<'a, 'b> {
    /// The maximum width available for individual line boxes.
    ///
    /// Note that this is just a guideline, line boxes may overflow
    /// if they cannot be broken up.
    available_width: CSSPixels,
    font_metrics: FontMetrics<'b>,
    text: &'a str,
    is_finished: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct TextLine<'a> {
    pub text: &'a str,
    pub width: CSSPixels,
}

impl<'a, 'b> Iterator for LineBreakIterator<'a, 'b> {
    type Item = TextLine<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_finished {
            return None;
        }

        let mut previous_break_point = None;
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

            match (width > self.available_width, previous_break_point) {
                (false, _) => {
                    previous_break_point = Some((line, remainder, width));
                    continue;
                },
                (true, None) => {
                    // Our line is too wide, but there was no opportunity to split it.
                    self.text = remainder;
                    return Some(TextLine { text: line, width });
                },
                (true, Some((line, remainder, width))) => {
                    self.text = remainder;
                    return Some(TextLine { text: line, width });
                },
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
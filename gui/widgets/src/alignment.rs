#[derive(Clone, Copy, Debug, Default)]
/// Describe the alignment of elements within a container
pub enum Alignment {
    /// Put all elements at the start of the container
    /// ```text,ignore
    /// -------------------------------------------------------
    /// | 1 | 2 | 3 | 4 |                                     |
    /// -------------------------------------------------------
    Start,
    /// Put all the elements at the end of the container
    /// ```text,ignore
    /// -------------------------------------------------------
    /// |                                     | 1 | 2 | 3 | 4 |
    /// -------------------------------------------------------
    End,
    /// Put all the elements in the center of the container
    /// ```text,ignore
    /// -------------------------------------------------------
    /// |                 | 1 | 2 | 3 | 4 |                   |
    /// -------------------------------------------------------
    Center,
    #[default]
    /// Evenly distribute the elements along the container
    /// ```text,ignore
    /// -------------------------------------------------------
    /// |      | 1 |       | 2 |       | 3 |       | 4 |      |
    /// -------------------------------------------------------
    Fill,
}

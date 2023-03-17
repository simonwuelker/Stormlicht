//! <https://infra.spec.whatwg.org>

/// <https://infra.spec.whatwg.org/#namespaces>
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Namespace {
    /// <https://infra.spec.whatwg.org/#html-namespace>
    HTML,

    /// <https://infra.spec.whatwg.org/#mathml-namespace>
    MathML,

    /// <https://infra.spec.whatwg.org/#svg-namespace>
    SVG,

    /// <https://infra.spec.whatwg.org/#xlink-namespace>
    XLink,

    /// <https://infra.spec.whatwg.org/#xml-namespace>
    XML,

    /// <https://infra.spec.whatwg.org/#xmlns-namespace>
    XMLNS,
}

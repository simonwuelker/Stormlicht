//! <https://infra.spec.whatwg.org>

use std::thread;

/// <https://infra.spec.whatwg.org/#namespaces>
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum Namespace {
    /// <https://infra.spec.whatwg.org/#html-namespace>
    #[default]
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

/// <https://infra.spec.whatwg.org/#normalize-newlines>
pub fn normalize_newlines(source: &str) -> String {
    source.replace("\r\n", "\n").replace('\r', "\n")
}

/// <https://html.spec.whatwg.org/multipage/infrastructure.html#in-parallel>
pub fn in_parallel<F, T>(f: F, task_name: Option<String>) -> thread::JoinHandle<T>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    let mut builder = thread::Builder::new();

    if let Some(name) = task_name {
        builder = builder.name(name)
    }

    builder.spawn(f).expect("Failed to spawn parallel task")
}

/// <https://262.ecma-international.org/14.0/#sec-completion-record-specification-type>
pub type ThrowCompletionOr<T> = Result<T, Exception>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Exception {
    TypeError,
}

//! <https://drafts.csswg.org/selectors/#grammar>

use crate::tree::ComponentValue;

/// <https://drafts.csswg.org/selectors/#typedef-selector-list>
pub type SelectorList = ComplexSelectorList;

/// <https://drafts.csswg.org/selectors/#typedef-complex-selector-list>
pub type ComplexSelectorList = Vec<ComplexSelector>;

/// <https://drafts.csswg.org/selectors/#typedef-complex-real-selector-list>
pub type ComplexRealSelectorList = Vec<ComplexRealSelector>;

/// <https://drafts.csswg.org/selectors/#typedef-compound-selector-list>
pub type CompoundSelectorList = Vec<CompoundSelector>;

/// <https://drafts.csswg.org/selectors/#typedef-simple-selector-list>
pub type SimpleSelectorList = Vec<SimpleSelector>;

/// <https://drafts.csswg.org/selectors/#typedef-relative-selector-list>
pub type RelativeSelectorList = Vec<RelativeSelector>;

/// <https://drafts.csswg.org/selectors/#typedef-relative-real-selector-list>
pub type RelativeRealSelectorList = Vec<RelativeRealSelector>;

/// <https://drafts.csswg.org/selectors/#typedef-complex-selector>
#[derive(Clone, Debug)]
pub struct ComplexSelector {
    pub first_unit: ComplexSelectorUnit,
    pub other_units: Vec<(Option<Combinator>, ComplexSelectorUnit)>,
}

/// <https://drafts.csswg.org/selectors/#typedef-complex-selector-unit>
#[derive(Clone, Debug)]
pub struct ComplexSelectorUnit {
    pub selectors: Vec<(Option<CompoundSelector>, Vec<PseudoCompoundSelector>)>,
}

/// <https://drafts.csswg.org/selectors/#typedef-complex-real-selector>
#[derive(Clone, Debug)]
pub struct ComplexRealSelector {
    pub first_selector: CompoundSelector,
    pub other_selectors: Vec<(Option<Combinator>, CompoundSelector)>,
}

/// <https://drafts.csswg.org/selectors/#typedef-relative-selector>
#[derive(Clone, Debug)]
pub struct RelativeSelector {
    pub combinator: Option<Combinator>,
    pub selector: ComplexSelector,
}

/// <https://drafts.csswg.org/selectors/#typedef-relative-real-selector>
#[derive(Clone, Debug)]
pub struct RelativeRealSelector {
    pub combinator: Option<Combinator>,
    pub selector: ComplexRealSelector,
}

/// <https://drafts.csswg.org/selectors/#typedef-compound-selector>
#[derive(Clone, Debug)]
pub struct CompoundSelector {
    pub selectors: Vec<(Option<TypeSelector>, Vec<SubclassSelector>)>,
}

/// <https://drafts.csswg.org/selectors/#typedef-pseudo-compound-selector>
#[derive(Clone, Debug)]
pub struct PseudoCompoundSelector {
    pub element_selector: PseudoElementSelector,
    pub class_selectors: Vec<PseudoClassSelector>,
}

/// <https://drafts.csswg.org/selectors/#typedef-simple-selector>
#[derive(Clone, Debug)]
pub enum SimpleSelector {
    TypeSelector(TypeSelector),
    SubclassSelector(SubclassSelector),
}

/// <https://drafts.csswg.org/selectors/#typedef-combinator>
#[derive(Clone, Copy, Debug)]
pub enum Combinator {
    /// `>`
    Child,
    /// `+`
    NextSibling,
    /// `~`
    SubsequentSibling,
    /// `||`
    DoubleOr, // TODO find better name, idk what this does
}

/// <https://drafts.csswg.org/selectors/#typedef-wq-name>
#[derive(Clone, Debug)]
pub struct WQName {
    pub prefix: Option<NSPrefix>,
    pub ident: String,
}

/// <https://drafts.csswg.org/selectors/#typedef-ns-prefix>
#[derive(Clone, Debug)]
pub enum NSPrefix {
    Ident(String),
    Asterisk,
    Nothing,
}

/// <https://drafts.csswg.org/selectors/#typedef-type-selector>
#[derive(Clone, Debug)]
pub enum TypeSelector {
    WQName(WQName),
    NSPrefix(NSPrefix),
    NoNSPrefix,
}

/// <https://drafts.csswg.org/selectors/#typedef-subclass-selector>
#[derive(Clone, Debug)]
pub enum SubclassSelector {
    Id(IdSelector),
    Class(ClassSelector),
    Attribute(AttributeSelector),
    PseudoClass(PseudoClassSelector),
}

/// <https://drafts.csswg.org/selectors/#typedef-id-selector>
#[derive(Clone, Debug)]
pub struct IdSelector(String);

/// <https://drafts.csswg.org/selectors/#typedef-class-selector>
#[derive(Clone, Debug)]
pub struct ClassSelector(String);

/// <https://drafts.csswg.org/selectors/#typedef-attribute-selector>
#[derive(Clone, Debug)]
pub enum AttributeSelector {
    ByName(WQName),
    ByValue {
        name: WQName,
        matcher: AttributeMatcher,
        value: String,
        modifier: AttributeModifier,
    },
}

/// <https://drafts.csswg.org/selectors/#typedef-attr-matcher>
#[derive(Clone, Copy, Debug)]
pub enum AttributeMatcher {
    /// `~=`
    Tilde,
    /// `|=`
    Bar,
    /// `^=`
    Caret,
    /// `$=`
    Dollar,
    /// `*=`
    Asterisk,
    /// `=`
    Default,
}

/// <https://drafts.csswg.org/selectors/#typedef-attr-modifier>
#[derive(Clone, Copy, Debug)]
pub enum AttributeModifier {
    /// `i`
    I,
    /// `s`
    S,
}

/// <https://drafts.csswg.org/selectors/#typedef-pseudo-class-selector>
#[derive(Clone, Debug)]
pub enum PseudoClassSelector {
    Name(String),
    Function(String, ()),
}

/// <https://drafts.csswg.org/selectors/#typedef-pseudo-element-selector>
#[derive(Clone, Debug)]
pub enum PseudoElementSelector {
    PseudoClass(PseudoClassSelector),
    Legacy(LegacyPseudoElementSelector),
}

/// <https://drafts.csswg.org/selectors/#typedef-legacy-pseudo-element-selector>
#[derive(Clone, Copy, Debug)]
pub enum LegacyPseudoElementSelector {
    /// `:before`
    Before,
    /// `:after`
    After,
    /// `:first-line`
    FirstLine,
    /// `:first-letter`
    FirstLetter,
}

pub trait CssGrammar: Sized {
    type ParseError = ();

    fn parse(component_values: &[ComponentValue]) -> Result<Self, Self::ParseError>;
}

impl CssGrammar for ComplexSelectorList {
    fn parse(_component_values: &[ComponentValue]) -> Result<Self, Self::ParseError> {
        todo!()
    }
}

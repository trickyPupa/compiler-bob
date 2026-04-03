#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SymbolInfo {
    pub(crate) name: String,
    pub(crate) is_initialized: bool,
    pub(crate) is_used: bool,
}
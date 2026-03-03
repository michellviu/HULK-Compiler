use std::fmt;

/// Internal representation of types in the HULK type system.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HulkType {
    /// Built-in numeric type (IEEE 754 f64).
    Number,
    /// Built-in string type.
    String,
    /// Built-in boolean type.
    Boolean,
    /// The void/unit type — result of `print`, `while` without else, etc.
    Void,
    /// A user-defined or built-in class type, identified by name.
    Class(std::string::String),
    /// Array of elements with a given element type.
    Array(Box<HulkType>),
    /// The `self` pseudo-type used inside method bodies.
    SelfType,
    /// Sentinel type used for error recovery — conforms to everything.
    Error,
    /// Type not yet determined (for inference).
    Unknown,
    /// The root type — every type conforms to Object.
    Object,
}

impl HulkType {
    /// Returns `true` if this is the error sentinel (used for recovery).
    pub fn is_error(&self) -> bool {
        matches!(self, HulkType::Error)
    }

    /// Returns `true` if this is a concrete (fully resolved) type.
    pub fn is_resolved(&self) -> bool {
        !matches!(self, HulkType::Unknown | HulkType::Error)
    }

    /// Returns the human-readable name of this type.
    pub fn type_name(&self) -> std::string::String {
        match self {
            HulkType::Number => "Number".into(),
            HulkType::String => "String".into(),
            HulkType::Boolean => "Boolean".into(),
            HulkType::Void => "Void".into(),
            HulkType::Object => "Object".into(),
            HulkType::Class(name) => name.clone(),
            HulkType::Array(inner) => format!("{}[]", inner.type_name()),
            HulkType::SelfType => "Self".into(),
            HulkType::Error => "<error>".into(),
            HulkType::Unknown => "<unknown>".into(),
        }
    }

    /// Converts a type-annotation string from the AST into a HulkType.
    pub fn from_name(name: &str) -> HulkType {
        match name {
            "Number" => HulkType::Number,
            "String" => HulkType::String,
            "Boolean" | "Bool" => HulkType::Boolean,
            "Void" => HulkType::Void,
            "Object" => HulkType::Object,
            other => HulkType::Class(other.to_string()),
        }
    }
}

impl fmt::Display for HulkType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.type_name())
    }
}

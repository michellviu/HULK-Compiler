/// A parameter with optional type annotation: `name [: Type]`
#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub type_ann: Option<String>,
}


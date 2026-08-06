/// One name binding in a scope: `(name, value, was_omitted)`. `was_omitted`
/// marks a lambda parameter that the caller didn't supply (for `ISOMITTED`).
type Binding = (String, Value, bool);

/// 对应 Java：无直接对应对象；Rust 架构扩展。 A per-evaluation cursor over a workbook snapshot.
pub struct Evaluator<'a> {
    pub wb: &'a Workbook,
    pub registry: &'a Registry,
    pub current: CellRef,
    pub now: f64,
    pub today: f64,
    /// Recursion guard against pathological/mutually recursive named formulas.
    depth: u32,
    /// Lexical scopes for `LET` bindings and `LAMBDA` parameters (innermost last).
    scopes: Vec<Vec<Binding>>,
}

const MAX_DEPTH: u32 = 256;

/// Functions that must receive raw reference arguments because they inspect a
/// cell's location/identity/geometry rather than its value. Every other function
/// has single-cell references dereferenced to scalars before it runs.
fn wants_reference(name: &str) -> bool {
    matches!(
        name,
        "ROW"
            | "COLUMN"
            | "ROWS"
            | "COLUMNS"
            | "AREAS"
            | "OFFSET"
            | "INDEX"
            | "CELL"
            | "SHEET"
            | "SHEETS"
            | "ISREF"
            | "ISFORMULA"
            | "FORMULATEXT"
    )
}


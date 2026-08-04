//! Formula lexer + Pratt parser. Produces the [`Expr`] AST from formula text
//! (with or without a leading `=`).

use super::ast::{BinaryOp, Expr, Reference, SheetSpec, UnaryOp};
use crate::core::addr::{CellAddress, col_letters_to_index};
use crate::core::error::CellError;

/// Maximum row/column indices for full-column / full-row references (XLSX grid).
const MAX_ROW: u32 = 1_048_575;
const MAX_COL: u32 = 16_383;

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Num(f64),
    Str(String),
    ErrLit(CellError),
    /// Unqualified identifier: function name, defined name, boolean, or cell ref.
    Ident(String),
    /// Sheet-qualified reference: `Sheet1!A1`, `'My Sheet'!B2`, `S1:S3!A1`.
    QRef {
        sheet: SheetSpec,
        body: String,
    },
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Semi,
    Colon,
    At,
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    Amp,
    Percent,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

/// A token plus whether whitespace preceded it (needed for the intersection op).
#[derive(Debug, Clone)]
struct Spanned {
    tok: Tok,
    ws_before: bool,
}

fn is_ref_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == '$' || c == '\\'
}

fn lex(input: &str) -> Result<Vec<Spanned>, String> {
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    let mut out = Vec::new();
    let mut ws = false;
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            ws = true;
            i += 1;
            continue;
        }
        let start_ws = ws;
        ws = false;
        let tok = match c {
            '(' => {
                i += 1;
                Tok::LParen
            }
            ')' => {
                i += 1;
                Tok::RParen
            }
            '{' => {
                i += 1;
                Tok::LBrace
            }
            '}' => {
                i += 1;
                Tok::RBrace
            }
            ',' => {
                i += 1;
                Tok::Comma
            }
            ';' => {
                i += 1;
                Tok::Semi
            }
            ':' => {
                i += 1;
                Tok::Colon
            }
            '@' => {
                i += 1;
                Tok::At
            }
            '+' => {
                i += 1;
                Tok::Plus
            }
            '-' => {
                i += 1;
                Tok::Minus
            }
            '*' => {
                i += 1;
                Tok::Star
            }
            '/' => {
                i += 1;
                Tok::Slash
            }
            '^' => {
                i += 1;
                Tok::Caret
            }
            '&' => {
                i += 1;
                Tok::Amp
            }
            '%' => {
                i += 1;
                Tok::Percent
            }
            '=' => {
                i += 1;
                Tok::Eq
            }
            '<' => {
                i += 1;
                if chars.get(i) == Some(&'=') {
                    i += 1;
                    Tok::Le
                } else if chars.get(i) == Some(&'>') {
                    i += 1;
                    Tok::Ne
                } else {
                    Tok::Lt
                }
            }
            '>' => {
                i += 1;
                if chars.get(i) == Some(&'=') {
                    i += 1;
                    Tok::Ge
                } else {
                    Tok::Gt
                }
            }
            '"' => {
                i += 1;
                let mut s = String::new();
                loop {
                    match chars.get(i) {
                        None => return Err("unterminated string".into()),
                        Some('"') => {
                            if chars.get(i + 1) == Some(&'"') {
                                s.push('"');
                                i += 2;
                            } else {
                                i += 1;
                                break;
                            }
                        }
                        Some(&ch) => {
                            s.push(ch);
                            i += 1;
                        }
                    }
                }
                Tok::Str(s)
            }
            '#' => {
                // error literal, e.g. #DIV/0! #N/A #REF!
                let rest: String = chars[i..].iter().collect();
                let (err, len) = parse_error_literal(&rest)?;
                i += len;
                Tok::ErrLit(err)
            }
            '\'' => {
                // quoted sheet name → must be followed by ! and a reference body
                let (sheet_name, consumed) = read_quoted_name(&chars, i)?;
                i += consumed;
                lex_after_sheet(&chars, &mut i, SheetSpec::Name(sheet_name))?
            }
            c if c.is_ascii_digit()
                || (c == '.' && chars.get(i + 1).is_some_and(|d| d.is_ascii_digit())) =>
            {
                let (n, len) = read_number(&chars, i);
                i += len;
                Tok::Num(n)
            }
            c if is_ref_char(c) => {
                // Read a run of reference characters.
                let start = i;
                while i < chars.len() && is_ref_char(chars[i]) {
                    i += 1;
                }
                let mut word: String = chars[start..i].iter().collect();
                // Structured (table) reference?  Table[...]  — absorb the
                // balanced `[...]` so it lexes as one identifier the evaluator
                // resolves against the workbook's tables (e.g. `Sales[Amount]`).
                if chars.get(i) == Some(&'[') {
                    let mut depth = 0usize;
                    while i < chars.len() {
                        let ch = chars[i];
                        word.push(ch);
                        i += 1;
                        match ch {
                            '[' => depth += 1,
                            ']' => {
                                depth -= 1;
                                if depth == 0 {
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                    if depth != 0 {
                        return Err("unterminated structured reference".into());
                    }
                    Tok::Ident(word)
                } else if chars.get(i) == Some(&'!') {
                    i += 1; // consume !
                    let body = read_ref_body(&chars, &mut i);
                    Tok::QRef {
                        sheet: SheetSpec::Name(word),
                        body,
                    }
                } else if chars.get(i) == Some(&':') && is_3d_span(&chars, i) {
                    i += 1; // consume :
                    let start2 = i;
                    while i < chars.len() && is_ref_char(chars[i]) {
                        i += 1;
                    }
                    let word2: String = chars[start2..i].iter().collect();
                    i += 1; // consume !
                    let body = read_ref_body(&chars, &mut i);
                    Tok::QRef {
                        sheet: SheetSpec::Span(word, word2),
                        body,
                    }
                } else {
                    Tok::Ident(word)
                }
            }
            other => return Err(format!("unexpected character '{other}'")),
        };
        out.push(Spanned {
            tok,
            ws_before: start_ws,
        });
    }
    Ok(out)
}

/// After a quoted sheet name `'..'`, expect `!` and read the reference body.
fn lex_after_sheet(chars: &[char], i: &mut usize, sheet: SheetSpec) -> Result<Tok, String> {
    // handle 3D quoted span 'A':'B'!  — rare; support simple 'A'!ref form.
    if chars.get(*i) == Some(&':') && chars.get(*i + 1) == Some(&'\'') {
        let (name2, consumed) = read_quoted_name(chars, *i + 1)?;
        *i += 1 + consumed;
        let first = match sheet {
            SheetSpec::Name(n) => n,
            _ => return Err("bad 3D sheet span".into()),
        };
        if chars.get(*i) != Some(&'!') {
            return Err("expected '!' after sheet span".into());
        }
        *i += 1;
        let body = read_ref_body(chars, i);
        return Ok(Tok::QRef {
            sheet: SheetSpec::Span(first, name2),
            body,
        });
    }
    if chars.get(*i) != Some(&'!') {
        return Err("expected '!' after quoted sheet name".into());
    }
    *i += 1;
    let body = read_ref_body(chars, i);
    Ok(Tok::QRef { sheet, body })
}

fn read_ref_body(chars: &[char], i: &mut usize) -> String {
    let start = *i;
    while *i < chars.len() && is_ref_char(chars[*i]) {
        *i += 1;
    }
    chars[start..*i].iter().collect()
}

/// Read a `'...'` quoted name starting at `start` (which must point at `'`).
/// Returns (name, chars_consumed including both quotes).
fn read_quoted_name(chars: &[char], start: usize) -> Result<(String, usize), String> {
    let mut i = start + 1;
    let mut s = String::new();
    loop {
        match chars.get(i) {
            None => return Err("unterminated sheet name".into()),
            Some('\'') => {
                if chars.get(i + 1) == Some(&'\'') {
                    s.push('\'');
                    i += 2;
                } else {
                    i += 1;
                    break;
                }
            }
            Some(&ch) => {
                s.push(ch);
                i += 1;
            }
        }
    }
    Ok((s, i - start))
}

/// Detect whether a `:` at position `i` is a 3D sheet span (`Name:Name!`) rather
/// than a range operator, by scanning ahead for `!` before any non-ref char.
fn is_3d_span(chars: &[char], colon: usize) -> bool {
    let mut j = colon + 1;
    if j >= chars.len() || !is_ref_char(chars[j]) {
        return false;
    }
    while j < chars.len() && is_ref_char(chars[j]) {
        j += 1;
    }
    chars.get(j) == Some(&'!')
}

fn read_number(chars: &[char], start: usize) -> (f64, usize) {
    let mut i = start;
    while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
        i += 1;
    }
    // exponent
    if i < chars.len() && (chars[i] == 'e' || chars[i] == 'E') {
        let mut j = i + 1;
        if chars.get(j) == Some(&'+') || chars.get(j) == Some(&'-') {
            j += 1;
        }
        if chars.get(j).is_some_and(|c| c.is_ascii_digit()) {
            i = j;
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
        }
    }
    let s: String = chars[start..i].iter().collect();
    (s.parse().unwrap_or(0.0), i - start)
}

fn parse_error_literal(s: &str) -> Result<(CellError, usize), String> {
    // Longest match among known error strings.
    const ERRS: &[(&str, CellError)] = &[
        ("#NULL!", CellError::Null),
        ("#DIV/0!", CellError::Div0),
        ("#VALUE!", CellError::Value),
        ("#REF!", CellError::Ref),
        ("#NAME?", CellError::Name),
        ("#NUM!", CellError::Num),
        ("#N/A", CellError::NA),
        ("#GETTING_DATA", CellError::GettingData),
        ("#SPILL!", CellError::Spill),
        ("#CALC!", CellError::Calc),
    ];
    let upper = s.to_ascii_uppercase();
    for (lit, err) in ERRS {
        if upper.starts_with(lit) {
            return Ok((*err, lit.len()));
        }
    }
    Err(format!(
        "unknown error literal at '{}'",
        &s[..s.len().min(8)]
    ))
}

struct Parser {
    toks: Vec<Spanned>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos).map(|s| &s.tok)
    }
    fn peek_ws(&self) -> bool {
        self.toks
            .get(self.pos)
            .map(|s| s.ws_before)
            .unwrap_or(false)
    }
    fn next(&mut self) -> Option<Tok> {
        let t = self.toks.get(self.pos).map(|s| s.tok.clone());
        self.pos += 1;
        t
    }

    fn parse_expr(&mut self, min_bp: u8) -> Result<Expr, String> {
        let mut lhs = self.parse_prefix()?;
        while let Some(op) = self.peek_binary_op() {
            let bp = op.precedence();
            if bp < min_bp {
                break;
            }
            // consume the operator token(s)
            self.consume_binary_op(op);
            let next_min = if op.left_assoc() { bp + 1 } else { bp };
            let rhs = self.parse_expr(next_min)?;
            lhs = combine(op, lhs, rhs);
        }
        Ok(lhs)
    }

    /// Peek at the upcoming binary operator, including the implicit intersection
    /// (a space between two operands).
    fn peek_binary_op(&self) -> Option<BinaryOp> {
        match self.peek()? {
            Tok::Plus => Some(BinaryOp::Add),
            Tok::Minus => Some(BinaryOp::Sub),
            Tok::Star => Some(BinaryOp::Mul),
            Tok::Slash => Some(BinaryOp::Div),
            Tok::Caret => Some(BinaryOp::Pow),
            Tok::Amp => Some(BinaryOp::Concat),
            Tok::Eq => Some(BinaryOp::Eq),
            Tok::Ne => Some(BinaryOp::Ne),
            Tok::Lt => Some(BinaryOp::Lt),
            Tok::Le => Some(BinaryOp::Le),
            Tok::Gt => Some(BinaryOp::Gt),
            Tok::Ge => Some(BinaryOp::Ge),
            Tok::Colon => Some(BinaryOp::Range),
            // Note: `,` is NOT a general binary operator here — it separates
            // function arguments. The union operator is only recognized inside an
            // explicit parenthesized group (see the `LParen` primary).
            // intersection: a value-starting token preceded by whitespace
            t if self.peek_ws() && starts_operand(t) => Some(BinaryOp::Intersect),
            _ => None,
        }
    }

    fn consume_binary_op(&mut self, op: BinaryOp) {
        // Intersection consumes no token (it's the space); everything else is one
        // or two characters already merged into a single Tok.
        if op != BinaryOp::Intersect {
            self.pos += 1;
        }
    }

    fn parse_prefix(&mut self) -> Result<Expr, String> {
        match self.peek() {
            Some(Tok::Minus) => {
                self.pos += 1;
                let e = self.parse_expr(6)?; // unary binds tighter than ^? Excel: -2^2 = 4
                Ok(Expr::Unary {
                    op: UnaryOp::Neg,
                    expr: Box::new(e),
                })
            }
            Some(Tok::Plus) => {
                self.pos += 1;
                let e = self.parse_expr(6)?;
                Ok(Expr::Unary {
                    op: UnaryOp::Plus,
                    expr: Box::new(e),
                })
            }
            Some(Tok::At) => {
                self.pos += 1;
                let e = self.parse_prefix()?;
                Ok(Expr::Func {
                    name: "_AT_".to_string(),
                    args: vec![e],
                })
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<Expr, String> {
        let mut e = self.parse_primary()?;
        while let Some(Tok::Percent) = self.peek() {
            self.pos += 1;
            e = Expr::Unary {
                op: UnaryOp::Percent,
                expr: Box::new(e),
            };
        }
        Ok(e)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        let tok = self.next().ok_or("unexpected end of formula")?;
        match tok {
            Tok::Num(n) => Ok(Expr::Number(n)),
            Tok::Str(s) => Ok(Expr::Text(s)),
            Tok::ErrLit(e) => Ok(Expr::Error(e)),
            Tok::LParen => {
                let mut e = self.parse_expr(0)?;
                // A parenthesized comma list is a reference union: (A1,B2,C3).
                while matches!(self.peek(), Some(Tok::Comma)) {
                    self.pos += 1;
                    let next = self.parse_expr(0)?;
                    e = Expr::Binary {
                        op: BinaryOp::Union,
                        lhs: Box::new(e),
                        rhs: Box::new(next),
                    };
                }
                match self.next() {
                    Some(Tok::RParen) => Ok(e),
                    _ => Err("expected ')'".into()),
                }
            }
            Tok::LBrace => self.parse_array_const(),
            Tok::Ident(word) => self.classify_ident(word),
            Tok::QRef { sheet, body } => self.make_qref(sheet, body),
            other => Err(format!("unexpected token {other:?}")),
        }
    }

    fn parse_array_const(&mut self) -> Result<Expr, String> {
        let mut rows: Vec<Vec<Expr>> = vec![Vec::new()];
        loop {
            match self.peek() {
                Some(Tok::RBrace) => {
                    self.pos += 1;
                    break;
                }
                _ => {
                    let e = self.parse_expr(0)?; // stops at the , / ; separators
                    rows.last_mut().unwrap().push(e);
                    match self.peek() {
                        Some(Tok::Comma) => {
                            self.pos += 1;
                        }
                        Some(Tok::Semi) => {
                            self.pos += 1;
                            rows.push(Vec::new());
                        }
                        Some(Tok::RBrace) => {
                            self.pos += 1;
                            break;
                        }
                        _ => return Err("malformed array constant".into()),
                    }
                }
            }
        }
        Ok(Expr::Array(rows))
    }

    fn classify_ident(&mut self, word: String) -> Result<Expr, String> {
        // Function call?
        if matches!(self.peek(), Some(Tok::LParen)) {
            self.pos += 1;
            let args = self.parse_args()?;
            return Ok(Expr::Func {
                name: word.to_ascii_uppercase(),
                args,
            });
        }
        // Boolean literal?
        if word.eq_ignore_ascii_case("TRUE") {
            return Ok(Expr::Bool(true));
        }
        if word.eq_ignore_ascii_case("FALSE") {
            return Ok(Expr::Bool(false));
        }
        // Cell reference?
        if let Some(addr) = CellAddress::parse_a1(&word) {
            return self.finish_ref(SheetSpec::Current, RefAtom::Cell(addr));
        }
        // Column-only / row-only token: only a reference when it begins a range
        // (`A:A`, `2:2`). A bare column letter on its own (`x`, `a`) is a name —
        // crucial so LAMBDA/LET parameters that look like column letters bind
        // correctly rather than parsing as whole-column references.
        if matches!(self.peek(), Some(Tok::Colon))
            && let Some(atom) = parse_col_or_row(&word)
        {
            return self.finish_ref(SheetSpec::Current, atom);
        }
        // Otherwise a defined name (or a LET/lambda-bound variable).
        Ok(Expr::Name(word))
    }

    fn make_qref(&mut self, sheet: SheetSpec, body: String) -> Result<Expr, String> {
        if let Some(addr) = CellAddress::parse_a1(&body) {
            return self.finish_ref(sheet, RefAtom::Cell(addr));
        }
        if let Some(atom) = parse_col_or_row(&body) {
            return self.finish_ref(sheet, atom);
        }
        // Sheet-qualified defined name (rare): treat as a plain name.
        Ok(Expr::Name(body))
    }

    /// Given a starting reference atom, optionally consume `:atom` to form a range.
    fn finish_ref(&mut self, sheet: SheetSpec, first: RefAtom) -> Result<Expr, String> {
        if matches!(self.peek(), Some(Tok::Colon)) {
            // Look ahead: is the next token another reference atom?
            if let Some(second_tok) = self.toks.get(self.pos + 1).map(|s| s.tok.clone())
                && let Some(second) = atom_from_tok(&second_tok)
            {
                self.pos += 2; // consume ':' and the atom
                let (start, end) = resolve_range(first, second);
                return Ok(Expr::Ref(Reference::range(sheet, start, end)));
            }
        }
        // single
        let addr = first.to_address_start();
        if let RefAtom::Cell(_) = first {
            Ok(Expr::Ref(Reference::cell(sheet, addr)))
        } else {
            // full column/row used alone → a range spanning it
            let (start, end) = full_span(first);
            Ok(Expr::Ref(Reference::range(sheet, start, end)))
        }
    }

    fn parse_args(&mut self) -> Result<Vec<Expr>, String> {
        let mut args = Vec::new();
        if matches!(self.peek(), Some(Tok::RParen)) {
            self.pos += 1;
            return Ok(args);
        }
        loop {
            // allow omitted arguments (e.g. IF(A1,,0)) as Empty
            if matches!(self.peek(), Some(Tok::Comma) | Some(Tok::RParen)) {
                args.push(Expr::Text(String::new())); // placeholder empty arg
            } else {
                // `,` is not an operator, so parse_expr stops at the separator.
                args.push(self.parse_expr(0)?);
            }
            match self.next() {
                Some(Tok::Comma) => continue,
                Some(Tok::RParen) => break,
                _ => return Err("expected ',' or ')' in arguments".into()),
            }
        }
        Ok(args)
    }
}

/// A reference atom: a concrete cell, or a bare column / row used in a range.
#[derive(Debug, Clone, Copy)]
enum RefAtom {
    Cell(CellAddress),
    Col(u32, bool),
    Row(u32, bool),
}

impl RefAtom {
    fn to_address_start(self) -> CellAddress {
        match self {
            RefAtom::Cell(a) => a,
            RefAtom::Col(c, abs) => CellAddress {
                row: 0,
                col: c,
                abs_row: false,
                abs_col: abs,
            },
            RefAtom::Row(r, abs) => CellAddress {
                row: r,
                col: 0,
                abs_row: abs,
                abs_col: false,
            },
        }
    }
}

fn atom_from_tok(tok: &Tok) -> Option<RefAtom> {
    match tok {
        Tok::Ident(w) => {
            if let Some(a) = CellAddress::parse_a1(w) {
                Some(RefAtom::Cell(a))
            } else {
                parse_col_or_row(w)
            }
        }
        Tok::QRef { body, .. } => {
            if let Some(a) = CellAddress::parse_a1(body) {
                Some(RefAtom::Cell(a))
            } else {
                parse_col_or_row(body)
            }
        }
        _ => None,
    }
}

/// Parse a bare column (`A`, `$AB`) or row (`5`, `$5`) used in a full-range ref.
fn parse_col_or_row(s: &str) -> Option<RefAtom> {
    let (abs, rest) = match s.strip_prefix('$') {
        Some(r) => (true, r),
        None => (false, s),
    };
    if !rest.is_empty() && rest.chars().all(|c| c.is_ascii_alphabetic()) {
        return col_letters_to_index(rest).map(|c| RefAtom::Col(c, abs));
    }
    if !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()) {
        let r: u32 = rest.parse().ok()?;
        if r >= 1 {
            return Some(RefAtom::Row(r - 1, abs));
        }
    }
    None
}

fn resolve_range(a: RefAtom, b: RefAtom) -> (CellAddress, CellAddress) {
    match (a, b) {
        (RefAtom::Cell(x), RefAtom::Cell(y)) => (x, y),
        (RefAtom::Col(c1, _), RefAtom::Col(c2, _)) => (
            CellAddress::new(0, c1.min(c2)),
            CellAddress::new(MAX_ROW, c1.max(c2)),
        ),
        (RefAtom::Row(r1, _), RefAtom::Row(r2, _)) => (
            CellAddress::new(r1.min(r2), 0),
            CellAddress::new(r1.max(r2), MAX_COL),
        ),
        // mixed / cell-to-line: best effort, take bounding box
        _ => {
            let sa = a.to_address_start();
            let sb = b.to_address_start();
            (sa, sb)
        }
    }
}

fn full_span(atom: RefAtom) -> (CellAddress, CellAddress) {
    match atom {
        RefAtom::Col(c, _) => (CellAddress::new(0, c), CellAddress::new(MAX_ROW, c)),
        RefAtom::Row(r, _) => (CellAddress::new(r, 0), CellAddress::new(r, MAX_COL)),
        RefAtom::Cell(a) => (a, a),
    }
}

fn starts_operand(tok: &Tok) -> bool {
    matches!(
        tok,
        Tok::Num(_)
            | Tok::Str(_)
            | Tok::ErrLit(_)
            | Tok::Ident(_)
            | Tok::QRef { .. }
            | Tok::LParen
            | Tok::LBrace
    )
}

fn combine(op: BinaryOp, lhs: Expr, rhs: Expr) -> Expr {
    Expr::Binary {
        op,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    }
}

/// Parse a formula string (leading `=` optional) into an [`Expr`].
pub fn parse(input: &str) -> Result<Expr, CellError> {
    parse_detailed(input).map_err(|_| CellError::Name)
}

/// Parse with a descriptive error message.
pub fn parse_detailed(input: &str) -> Result<Expr, String> {
    let text = input.strip_prefix('=').unwrap_or(input).trim();
    if text.is_empty() {
        return Err("empty formula".into());
    }
    let toks = lex(text)?;
    if toks.is_empty() {
        return Err("empty formula".into());
    }
    let mut p = Parser { toks, pos: 0 };
    let e = p.parse_expr(0)?;
    if p.pos != p.toks.len() {
        return Err(format!("unexpected trailing tokens at {}", p.pos));
    }
    Ok(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> Expr {
        parse_detailed(s).unwrap_or_else(|e| panic!("parse {s:?}: {e}"))
    }

    #[test]
    fn literals() {
        assert_eq!(p("=42"), Expr::Number(42.0));
        assert_eq!(
            p("=-3.5"),
            Expr::Unary {
                op: UnaryOp::Neg,
                expr: Box::new(Expr::Number(3.5))
            }
        );
        assert_eq!(p(r#"="hi""#), Expr::Text("hi".into()));
        assert_eq!(p("=TRUE"), Expr::Bool(true));
        assert_eq!(p("=#REF!"), Expr::Error(CellError::Ref));
    }

    #[test]
    fn precedence() {
        // 1+2*3 → 1 + (2*3)
        let e = p("=1+2*3");
        if let Expr::Binary {
            op: BinaryOp::Add,
            rhs,
            ..
        } = e
        {
            assert!(matches!(
                *rhs,
                Expr::Binary {
                    op: BinaryOp::Mul,
                    ..
                }
            ));
        } else {
            panic!("bad tree");
        }
    }

    #[test]
    fn references() {
        assert!(matches!(p("=A1"), Expr::Ref(_)));
        assert!(matches!(p("=A1:B10"), Expr::Ref(r) if r.is_range()));
        match p("=Sheet1!A1") {
            Expr::Ref(r) => assert_eq!(r.sheet, SheetSpec::Name("Sheet1".into())),
            _ => panic!(),
        }
        match p("='My Sheet'!B2") {
            Expr::Ref(r) => assert_eq!(r.sheet, SheetSpec::Name("My Sheet".into())),
            _ => panic!(),
        }
    }

    #[test]
    fn func_calls() {
        match p("=SUM(A1:A3, 5)") {
            Expr::Func { name, args } => {
                assert_eq!(name, "SUM");
                assert_eq!(args.len(), 2);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn array_constant() {
        match p("={1,2;3,4}") {
            Expr::Array(rows) => {
                assert_eq!(rows.len(), 2);
                assert_eq!(rows[0].len(), 2);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn three_d_ref() {
        match p("=Sheet1:Sheet3!A1") {
            Expr::Ref(r) => assert_eq!(r.sheet, SheetSpec::Span("Sheet1".into(), "Sheet3".into())),
            _ => panic!(),
        }
    }

    #[test]
    fn full_column() {
        match p("=SUM(A:A)") {
            Expr::Func { args, .. } => match &args[0] {
                Expr::Ref(r) => {
                    assert_eq!(r.start.col, 0);
                    assert_eq!(r.end.unwrap().row, MAX_ROW);
                }
                _ => panic!(),
            },
            _ => panic!(),
        }
    }
}

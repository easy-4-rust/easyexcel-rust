impl Parser<'_> {
    fn peek(&self) -> Option<&LexTok> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<LexTok> {
        let tok = self.tokens.get(self.pos).cloned();
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }

    fn err(&self, detail: &str) -> ExcelError {
        format_error(self.formula, detail)
    }

    fn parse(&mut self) -> Result<Vec<RpnTok>, ExcelError> {
        self.parse_expr()?;
        if let Some(tok) = self.peek() {
            return Err(self.err(&format!("表达式结尾出现多余令牌 {tok:?}")));
        }
        Ok(std::mem::take(&mut self.out))
    }

    // expr := comparison
    fn parse_expr(&mut self) -> Result<(), ExcelError> {
        self.parse_compare()
    }

    // comparison := concat (comp_op concat)*
    fn parse_compare(&mut self) -> Result<(), ExcelError> {
        self.parse_concat()?;
        loop {
            match self.peek() {
                Some(LexTok::BinOp(op)) if matches!(op, 0x09..=0x0e) => {
                    let op = *op;
                    self.next();
                    self.parse_concat()?;
                    self.out.push(RpnTok::BinOp(op));
                }
                _ => return Ok(()),
            }
        }
    }

    // concat := additive ('&' additive)*
    fn parse_concat(&mut self) -> Result<(), ExcelError> {
        self.parse_additive()?;
        while matches!(self.peek(), Some(LexTok::BinOp(0x08))) {
            self.next();
            self.parse_additive()?;
            self.out.push(RpnTok::BinOp(0x08));
        }
        Ok(())
    }

    // additive := multiplicative (('+'|'-') multiplicative)*
    fn parse_additive(&mut self) -> Result<(), ExcelError> {
        self.parse_multiplicative()?;
        loop {
            match self.peek() {
                Some(LexTok::BinOp(op)) if matches!(op, 0x03 | 0x04) => {
                    let op = *op;
                    self.next();
                    self.parse_multiplicative()?;
                    self.out.push(RpnTok::BinOp(op));
                }
                _ => return Ok(()),
            }
        }
    }

    // multiplicative := power (('*'|'/') power)*
    fn parse_multiplicative(&mut self) -> Result<(), ExcelError> {
        self.parse_power()?;
        loop {
            match self.peek() {
                Some(LexTok::BinOp(op)) if matches!(op, 0x05 | 0x06) => {
                    let op = *op;
                    self.next();
                    self.parse_power()?;
                    self.out.push(RpnTok::BinOp(op));
                }
                _ => return Ok(()),
            }
        }
    }

    // power := unary ('^' power)?   （右结合）
    fn parse_power(&mut self) -> Result<(), ExcelError> {
        self.parse_unary()?;
        if matches!(self.peek(), Some(LexTok::BinOp(0x07))) {
            self.next();
            self.parse_power()?;
            self.out.push(RpnTok::BinOp(0x07));
        }
        Ok(())
    }

    // unary := ('-'|'+') unary | postfix
    fn parse_unary(&mut self) -> Result<(), ExcelError> {
        match self.peek() {
            Some(LexTok::UnaryOp(op)) => {
                let op = *op;
                self.next();
                self.parse_unary()?;
                self.out.push(RpnTok::UnaryOp(op));
                Ok(())
            }
            _ => self.parse_postfix(),
        }
    }

    // postfix := primary ('%')*
    fn parse_postfix(&mut self) -> Result<(), ExcelError> {
        self.parse_primary()?;
        while matches!(self.peek(), Some(LexTok::Percent)) {
            self.next();
            self.out.push(RpnTok::Percent);
        }
        Ok(())
    }

    // primary := number | string | bool | error | ref[:ref] | name(args) | (expr)
    fn parse_primary(&mut self) -> Result<(), ExcelError> {
        match self.next() {
            Some(LexTok::Number(v)) => {
                if v.fract() == 0.0 && (-32_768.0..=32_767.0).contains(&v) {
                    // 范围内必然可精确转换为 i16
                    #[allow(clippy::cast_possible_truncation)]
                    self.out.push(RpnTok::Int(v as i16));
                } else {
                    self.out.push(RpnTok::Num(v));
                }
                Ok(())
            }
            Some(LexTok::Str(s)) => {
                self.out.push(RpnTok::Str(s));
                Ok(())
            }
            Some(LexTok::Bool(b)) => {
                self.out.push(RpnTok::Bool(b));
                Ok(())
            }
            Some(LexTok::Err(code)) => {
                self.out.push(RpnTok::Err(code));
                Ok(())
            }
            Some(LexTok::Ref {
                row,
                col,
                row_rel,
                col_rel,
            }) => {
                if matches!(self.peek(), Some(LexTok::Colon)) {
                    self.next();
                    match self.next() {
                        Some(LexTok::Ref {
                            row: row2,
                            col: col2,
                            row_rel: row2_rel,
                            col_rel: col2_rel,
                        }) => {
                            self.out.push(RpnTok::Area(
                                row, row2, col, col2, row_rel, row2_rel, col_rel, col2_rel,
                            ));
                            Ok(())
                        }
                        Some(other) => {
                            Err(self.err(&format!("区域引用后应为单元格引用，得到 {other:?}")))
                        }
                        None => Err(self.err("区域引用缺少结束单元格")),
                    }
                } else {
                    self.out.push(RpnTok::Ref(row, col, row_rel, col_rel));
                    Ok(())
                }
            }
            Some(LexTok::Ref3d {
                first_sheet,
                last_sheet,
                row,
                col,
                row_rel,
                col_rel,
            }) => {
                if matches!(self.peek(), Some(LexTok::Colon)) {
                    self.next();
                    match self.next() {
                        Some(LexTok::Ref {
                            row: row2,
                            col: col2,
                            row_rel: row2_rel,
                            col_rel: col2_rel,
                        }) => {
                            self.out.push(RpnTok::Area3d(
                                first_sheet,
                                last_sheet,
                                row,
                                row2,
                                col,
                                col2,
                                row_rel,
                                row2_rel,
                                col_rel,
                                col2_rel,
                            ));
                            Ok(())
                        }
                        Some(LexTok::Ref3d {
                            first_sheet: second_first,
                            last_sheet: second_last,
                            row: row2,
                            col: col2,
                            row_rel: row2_rel,
                            col_rel: col2_rel,
                        }) if second_first.eq_ignore_ascii_case(&first_sheet)
                            && second_last.eq_ignore_ascii_case(&last_sheet) =>
                        {
                            self.out.push(RpnTok::Area3d(
                                first_sheet,
                                last_sheet,
                                row,
                                row2,
                                col,
                                col2,
                                row_rel,
                                row2_rel,
                                col_rel,
                                col2_rel,
                            ));
                            Ok(())
                        }
                        Some(other) => Err(self.err(&format!(
                            "3D 区域引用后应为同一工作表范围的单元格引用，得到 {other:?}"
                        ))),
                        None => Err(self.err("3D 区域引用缺少结束单元格")),
                    }
                } else {
                    self.out.push(RpnTok::Ref3d(
                        first_sheet,
                        last_sheet,
                        row,
                        col,
                        row_rel,
                        col_rel,
                    ));
                    Ok(())
                }
            }
            Some(LexTok::Name(name)) => {
                // 必须为函数调用
                if !matches!(self.peek(), Some(LexTok::LParen)) {
                    return Err(self.err(&format!(
                        "无法解析名称 {name}（命名区域暂不支持，请检查拼写）"
                    )));
                }
                let meta = find_function(&name).ok_or_else(|| {
                    self.err(&format!("未知函数 {name}（BIFF8 内建函数表中不存在）"))
                })?;
                self.next(); // '('
                let args = self.parse_args()?;
                if args < meta.1 || args > meta.2 {
                    return Err(self.err(&format!(
                        "函数 {name} 参数个数 {args} 超出范围 {}..={}",
                        meta.1, meta.2
                    )));
                }
                if meta.1 == meta.2 {
                    self.out.push(RpnTok::Func(0x21 + meta.3, meta.0));
                } else {
                    self.out.push(RpnTok::FuncVar(0x22 + meta.3, args, meta.0));
                }
                Ok(())
            }
            Some(LexTok::LParen) => {
                self.parse_expr()?;
                match self.next() {
                    Some(LexTok::RParen) => {
                        self.out.push(RpnTok::Paren);
                        Ok(())
                    }
                    _ => Err(self.err("括号不匹配：缺少 )")),
                }
            }
            Some(other) => Err(self.err(&format!("此处不允许出现 {other:?}"))),
            None => Err(self.err("表达式意外结束")),
        }
    }

    /// 解析函数参数列表（`(` 已消费，`)` 由调用方消费）。
    /// 空参数以 tMissArg 填充；返回参数个数。
    fn parse_args(&mut self) -> Result<u8, ExcelError> {
        let mut count = 0u8;
        loop {
            if matches!(self.peek(), Some(LexTok::RParen)) {
                self.next();
                return Ok(count);
            }
            if matches!(self.peek(), Some(LexTok::Comma)) {
                // 空参数
                self.next();
                self.out.push(RpnTok::MissArg);
                count += 1;
                continue;
            }
            self.parse_expr()?;
            count += 1;
            match self.peek() {
                Some(LexTok::Comma) => {
                    self.next();
                }
                Some(LexTok::RParen) => {
                    self.next();
                    return Ok(count);
                }
                _ => return Err(self.err("函数参数列表缺少 ) 或 ,")),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Ptg 字节编码
// ---------------------------------------------------------------------------

/// 把公式字符串编码为 BIFF8 FORMULA 记录 rgce（RPN Ptg 令牌数组）。
///
/// `formula` 可带或不带前导 `=`。
///
/// # Errors
///
/// 语法错误、未知函数或不受支持的引用风格时返回 [`ExcelError::Xls`]。
pub fn encode_formula_rpn(formula: &str) -> Result<Vec<u8>, ExcelError> {
    encode_formula_rpn_with_links(formula, None)
}

pub(super) fn encode_formula_rpn_with_link_table(
    formula: &str,
    links: &Biff8LinkTable,
) -> Result<Vec<u8>, ExcelError> {
    encode_formula_rpn_with_links(formula, Some(links))
}

fn encode_formula_rpn_with_links(
    formula: &str,
    links: Option<&Biff8LinkTable>,
) -> Result<Vec<u8>, ExcelError> {
    let expr = formula.strip_prefix('=').unwrap_or(formula);
    let tokens = tokenize(expr)?;
    let mut parser = Parser {
        tokens: &tokens,
        pos: 0,
        formula: expr,
        out: Vec::new(),
    };
    let rpn = parser.parse()?;
    let mut out = Vec::new();
    for tok in rpn {
        encode_token(&tok, &mut out, links)?;
    }
    Ok(out)
}

fn encode_token(
    tok: &RpnTok,
    out: &mut Vec<u8>,
    links: Option<&Biff8LinkTable>,
) -> Result<(), ExcelError> {
    match tok {
        RpnTok::Int(v) => {
            // tInt (0x1E)：2 字节有符号短整型
            out.push(0x1e);
            out.extend_from_slice(&v.to_le_bytes());
        }
        RpnTok::Num(v) => {
            // tNum (0x1F)：8 字节 IEEE754
            out.push(0x1f);
            out.extend_from_slice(&v.to_le_bytes());
        }
        RpnTok::Str(s) => {
            // tStr (0x17)：cch(1) + grbit(1) + 字符数据
            let chars: Vec<char> = s.chars().collect();
            if chars.len() > 255 {
                return Err(format_error(s, "公式字符串超过 255 字符"));
            }
            out.push(0x17);
            // 已检查 <= 255，不会截断
            #[allow(clippy::cast_possible_truncation)]
            out.push(chars.len() as u8);
            let compressed: Option<Vec<u8>> = chars
                .iter()
                .map(|c| u8::try_from(u32::from(*c)).ok())
                .collect();
            if let Some(bytes) = compressed {
                out.push(0x00);
                out.extend_from_slice(&bytes);
            } else {
                out.push(0x01);
                for c in chars {
                    out.extend_from_slice(&(c as u32).to_le_bytes()[..2]);
                }
            }
        }
        RpnTok::Bool(b) => {
            out.push(0x1d);
            out.push(u8::from(*b));
        }
        RpnTok::Err(code) => {
            out.push(0x1c);
            out.push(*code);
        }
        RpnTok::MissArg => out.push(0x16),
        RpnTok::Ref(row, col, row_rel, col_rel) => {
            // tRef (0x24)：rw(2) + col(2)，相对标志在 col 字段
            out.push(0x24);
            out.extend_from_slice(&row.to_le_bytes());
            let mut col_field = *col;
            if *row_rel {
                col_field |= 0x8000;
            }
            if *col_rel {
                col_field |= 0x4000;
            }
            out.extend_from_slice(&col_field.to_le_bytes());
        }
        RpnTok::Ref3d(first_sheet, last_sheet, row, col, row_rel, col_rel) => {
            let links = links.ok_or_else(|| {
                format_error(first_sheet, "3D 引用需要工作簿级 LinkTable")
            })?;
            let ixti = links.ixti(first_sheet, last_sheet).ok_or_else(|| {
                format_error(first_sheet, "3D 引用未在工作簿 LinkTable 中注册")
            })?;
            out.push(0x3a);
            out.extend_from_slice(&ixti.to_le_bytes());
            out.extend_from_slice(&row.to_le_bytes());
            let mut col_field = *col;
            if *row_rel {
                col_field |= 0x8000;
            }
            if *col_rel {
                col_field |= 0x4000;
            }
            out.extend_from_slice(&col_field.to_le_bytes());
        }
        RpnTok::Area(
            rw_first,
            rw_last,
            col_first,
            col_last,
            rw_first_rel,
            rw_last_rel,
            col_first_rel,
            col_last_rel,
        ) => {
            // tArea (0x25)：rwFirst + rwLast + colFirst + colLast
            out.push(0x25);
            out.extend_from_slice(&rw_first.to_le_bytes());
            out.extend_from_slice(&rw_last.to_le_bytes());
            let mut cf = *col_first;
            if *rw_first_rel {
                cf |= 0x8000;
            }
            if *col_first_rel {
                cf |= 0x4000;
            }
            out.extend_from_slice(&cf.to_le_bytes());
            let mut cl = *col_last;
            if *rw_last_rel {
                cl |= 0x8000;
            }
            if *col_last_rel {
                cl |= 0x4000;
            }
            out.extend_from_slice(&cl.to_le_bytes());
        }
        RpnTok::Area3d(
            first_sheet,
            last_sheet,
            rw_first,
            rw_last,
            col_first,
            col_last,
            rw_first_rel,
            rw_last_rel,
            col_first_rel,
            col_last_rel,
        ) => {
            let links = links.ok_or_else(|| {
                format_error(first_sheet, "3D 区域引用需要工作簿级 LinkTable")
            })?;
            let ixti = links.ixti(first_sheet, last_sheet).ok_or_else(|| {
                format_error(first_sheet, "3D 区域引用未在工作簿 LinkTable 中注册")
            })?;
            out.push(0x3b);
            out.extend_from_slice(&ixti.to_le_bytes());
            out.extend_from_slice(&rw_first.to_le_bytes());
            out.extend_from_slice(&rw_last.to_le_bytes());
            let mut first_col = *col_first;
            if *rw_first_rel {
                first_col |= 0x8000;
            }
            if *col_first_rel {
                first_col |= 0x4000;
            }
            out.extend_from_slice(&first_col.to_le_bytes());
            let mut last_col = *col_last;
            if *rw_last_rel {
                last_col |= 0x8000;
            }
            if *col_last_rel {
                last_col |= 0x4000;
            }
            out.extend_from_slice(&last_col.to_le_bytes());
        }
        RpnTok::Func(base, ifunc) => {
            // tFunc (0x21/0x41/0x61)：ptg + UShort(ifunc)，3 字节
            out.push(*base);
            out.extend_from_slice(&ifunc.to_le_bytes());
        }
        RpnTok::FuncVar(base, args, ifunc) => {
            // tFuncVar (0x22/0x42/0x62)：ptg + cparams(1) + UShort(ifunc)，4 字节
            out.push(*base);
            out.push(*args);
            out.extend_from_slice(&ifunc.to_le_bytes());
        }
        RpnTok::BinOp(op) | RpnTok::UnaryOp(op) => out.push(*op),
        RpnTok::Paren => out.push(0x15),
        RpnTok::Percent => out.push(0x14),
    }
    Ok(())
}

#[cfg(test)]
#[path = "../ptg_tests/tests.rs"]
mod tests;

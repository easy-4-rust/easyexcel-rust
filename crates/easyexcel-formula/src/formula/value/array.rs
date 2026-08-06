/// 对应 Java：无直接对应对象；Rust 架构扩展。 A dense 2D array of scalar values (row-major). Used for array constants and
/// CSE array results. Elements are never themselves arrays or refs.
#[derive(Debug, Clone, PartialEq)]
pub struct Array {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<Value>,
}

impl Array {
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn new(rows: usize, cols: usize, data: Vec<Value>) -> Self {
        debug_assert_eq!(rows * cols, data.len());
        Array { rows, cols, data }
    }

    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn scalar(v: Value) -> Self {
        Array {
            rows: 1,
            cols: 1,
            data: vec![v],
        }
    }

    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn from_rows(rows: Vec<Vec<Value>>) -> Self {
        let nrows = rows.len();
        let ncols = rows.first().map_or(0, std::vec::Vec::len);
        let data = rows.into_iter().flatten().collect();
        Array {
            rows: nrows,
            cols: ncols,
            data,
        }
    }

    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn get(&self, r: usize, c: usize) -> Option<&Value> {
        if r < self.rows && c < self.cols {
            self.data.get(r * self.cols + c)
        } else {
            None
        }
    }
}


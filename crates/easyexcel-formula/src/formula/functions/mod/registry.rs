/// Case-insensitive function dispatch table.
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub struct Registry {
    map: HashMap<&'static str, FnEntry>,
}

impl Registry {
    fn new() -> Self {
        Registry {
            map: HashMap::new(),
        }
    }

    /// Register a function. Panics on duplicate names (a programming error).
    ///
    /// # Panics
    ///
    /// 注册表中已经存在同名函数时 panic；这表示标准库构建代码存在缺陷。
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn add(
        &mut self,
        name: &'static str,
        min: usize,
        max: usize,
        volatile: bool,
        func: FnImpl,
    ) {
        let entry = FnEntry {
            name,
            min_args: min,
            max_args: max,
            volatile,
            func,
        };
        assert!(
            self.map.insert(name, entry).is_none(),
            "duplicate function registration: {name}"
        );
    }

    /// Register `alias` as a thin synonym for an already-registered function.
    ///
    /// # Panics
    ///
    /// 目标函数不存在或别名重复时 panic；这表示标准库构建代码存在缺陷。
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn alias(&mut self, alias: &'static str, target: &'static str) {
        let mut entry = self
            .map
            .get(target)
            .unwrap_or_else(|| panic!("alias target not found: {target}"))
            .clone();
        entry.name = alias;
        assert!(
            self.map.insert(alias, entry).is_none(),
            "duplicate function registration: {alias}"
        );
    }

    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn get(&self, name: &str) -> Option<&FnEntry> {
        // Names are stored upper-case; look up with an upper-cased key. Strip the
        // OOXML `_xlfn.` future-function prefix if present.
        let n = name.trim_start_matches("_xlfn.").to_ascii_uppercase();
        self.map.get(n.as_str())
    }

    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn is_volatile(&self, name: &str) -> bool {
        self.get(name).is_some_and(|e| e.volatile)
    }

    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn len(&self) -> usize {
        self.map.len()
    }

    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Build the standard library registry.
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn standard() -> Self {
        let mut r = Registry::new();
        logical::register(&mut r);
        math::register(&mut r);
        text::register(&mut r);
        stats::register(&mut r);
        lookup::register(&mut r);
        datetime::register(&mut r);
        info::register(&mut r);
        financial::register(&mut r);
        engineering::register(&mut r);
        database::register(&mut r);
        dynamic::register(&mut r);
        stubs::register(&mut r);
        r
    }
}

impl Default for Registry {
    fn default() -> Self {
        Registry::standard()
    }
}


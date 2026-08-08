//! 对应 Java：`com.alibaba.excel.metadata.AbstractHolder`.

use crate::CacheLocation;
use crate::ConverterRegistry;
use crate::Holder as HolderEnum;

use super::basic_parameter::BasicParameter;
use super::configuration_holder::ConfigurationHolder;
use super::global_configuration::GlobalConfiguration;

/// 对应 Java：com.alibaba.excel.metadata.AbstractHolder。 Shared holder state for read and write pipelines.
///
/// Rust port of Java `AbstractHolder implements ConfigurationHolder`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbstractHolder {
    /// Whether the holder was created in this request. (Java `newInitialization`)
    pub new_initialization: bool,
    /// Dynamic header rows. (Java `head`)
    pub head: Option<Vec<Vec<String>>>,
    /// Model type name. (Java `clazz`)
    pub clazz: Option<String>,
    /// Global configuration. (Java `globalConfiguration`)
    pub global_configuration: GlobalConfiguration,
    /// Registered converters. (Java `converterMap`)
    pub converter_map: ConverterRegistry,
    /// Holder scope. (Java `holderType()` on concrete subclasses)
    pub holder_type: HolderEnum,
}

impl Default for AbstractHolder {
    fn default() -> Self {
        Self::new(HolderEnum::Workbook)
    }
}

impl AbstractHolder {
    /// 对应 Java：com.alibaba.excel.metadata.AbstractHolder。 Creates an empty workbook-scoped holder. (Java no-args constructor)
    #[must_use]
    pub fn new(holder_type: HolderEnum) -> Self {
        Self {
            new_initialization: true,
            head: None,
            clazz: None,
            global_configuration: GlobalConfiguration::new(),
            converter_map: ConverterRegistry::default(),
            holder_type,
        }
    }

    /// 对应 Java：com.alibaba.excel.metadata.AbstractHolder。 Initializes holder state from builder parameters and an optional parent.
    /// (Java `AbstractHolder(BasicParameter, AbstractHolder)`)
    #[must_use]
    pub fn from_parameter(
        basic_parameter: &BasicParameter,
        parent: Option<&AbstractHolder>,
        holder_type: HolderEnum,
    ) -> Self {
        let mut holder = Self::new(holder_type);
        holder.new_initialization = true;

        if basic_parameter.head.is_none()
            && basic_parameter.clazz.is_none()
            && let Some(parent) = parent
        {
            holder.head.clone_from(&parent.head);
        } else {
            holder.head.clone_from(&basic_parameter.head);
        }

        if basic_parameter.head.is_none()
            && basic_parameter.clazz.is_none()
            && let Some(parent) = parent
        {
            holder.clazz.clone_from(&parent.clazz);
        } else {
            holder.clazz.clone_from(&basic_parameter.clazz);
        }

        holder.global_configuration = GlobalConfiguration::new();
        holder.global_configuration.auto_trim = basic_parameter
            .auto_trim
            .or_else(|| parent.map(|parent| parent.global_configuration.auto_trim))
            .unwrap_or(true);
        holder.global_configuration.use1904windowing = basic_parameter
            .use1904windowing
            .or_else(|| parent.map(|parent| parent.global_configuration.use1904windowing))
            .unwrap_or(false);
        holder.global_configuration.locale = basic_parameter
            .locale
            .clone()
            .or_else(|| parent.map(|parent| parent.global_configuration.locale.clone()))
            .unwrap_or_else(|| "default".to_owned());
        holder.global_configuration.use_scientific_format = basic_parameter
            .use_scientific_format
            .or_else(|| parent.map(|parent| parent.global_configuration.use_scientific_format))
            .unwrap_or(false);
        holder.global_configuration.filed_cache_location = basic_parameter
            .filed_cache_location
            .or_else(|| parent.map(|parent| parent.global_configuration.filed_cache_location))
            .unwrap_or(CacheLocation::ThreadLocal);

        holder
    }

    /// 对应 Java：com.alibaba.excel.metadata.AbstractHolder。 Returns the dynamic header rows. (Java `getHead()`)
    #[must_use]
    pub fn head(&self) -> Option<&[Vec<String>]> {
        self.head.as_deref()
    }

    /// 对应 Java：com.alibaba.excel.metadata.AbstractHolder。 Returns the model type name. (Java `getClazz()`)
    #[must_use]
    pub fn clazz(&self) -> Option<&str> {
        self.clazz.as_deref()
    }

    /// Java `getNewInitialization`。
    #[must_use] pub const fn get_new_initialization(&self) -> bool { self.new_initialization }
    /// Java `setNewInitialization`。
    pub const fn set_new_initialization(&mut self, value: bool) { self.new_initialization = value; }
    /// Java `getHead` 别名。
    #[must_use] pub fn get_head(&self) -> Option<&[Vec<String>]> { self.head.as_deref() }
    /// Java `setHead`。
    pub fn set_head(&mut self, value: Option<Vec<Vec<String>>>) { self.head = value; }
    /// Java `getClazz` 的 Rust 类型名映射。
    #[must_use] pub fn get_clazz(&self) -> Option<&str> { self.clazz.as_deref() }
    /// Java `setClazz` 的 Rust 类型名映射。
    pub fn set_clazz(&mut self, value: Option<String>) { self.clazz = value; }
    /// Java `getGlobalConfiguration`。
    #[must_use] pub const fn get_global_configuration(&self) -> &GlobalConfiguration { &self.global_configuration }
    /// Java `setGlobalConfiguration`。
    pub fn set_global_configuration(&mut self, value: GlobalConfiguration) { self.global_configuration = value; }
    /// Java `getConverterMap`。
    #[must_use] pub const fn get_converter_map(&self) -> &ConverterRegistry { &self.converter_map }
    /// Java `setConverterMap`。
    pub fn set_converter_map(&mut self, value: ConverterRegistry) { self.converter_map = value; }
    /// Java `converterMap()`。
    #[must_use] pub const fn converter_map(&self) -> &ConverterRegistry { &self.converter_map }
    /// Java `globalConfiguration()`。
    #[must_use] pub const fn global_configuration(&self) -> &GlobalConfiguration { &self.global_configuration }
    /// Java `isNew()`。
    #[must_use] pub const fn is_new(&self) -> bool { self.new_initialization }
}

impl super::configuration_holder::MetadataHolder for AbstractHolder {
    fn holder_type(&self) -> HolderEnum {
        self.holder_type
    }
}

impl ConfigurationHolder for AbstractHolder {
    fn is_new(&self) -> bool {
        self.new_initialization
    }

    fn global_configuration(&self) -> &GlobalConfiguration {
        &self.global_configuration
    }

    fn converter_map(&self) -> &ConverterRegistry {
        &self.converter_map
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn default_delegates_to_new() {
        // 对应 Java：AbstractHolder 默认构造为 Workbook 作用域
        let holder = AbstractHolder::default();
        assert_eq!(holder, AbstractHolder::new(HolderEnum::Workbook));
        assert!(holder.new_initialization);
    }

    #[test]
    fn from_parameter_inherits_parent_head_and_clazz() {
        // 对应 Java：子 holder 未指定 head/clazz 时继承父级
        let parent = AbstractHolder {
            head: Some(vec![vec!["Name".to_owned()]]),
            clazz: Some("Model".to_owned()),
            ..AbstractHolder::new(HolderEnum::Workbook)
        };
        let parameter = BasicParameter::new();
        let child = AbstractHolder::from_parameter(&parameter, Some(&parent), HolderEnum::Sheet);
        assert_eq!(child.head(), Some(&[vec!["Name".to_owned()]][..]));
        assert_eq!(child.clazz(), Some("Model"));
        assert_eq!(child.holder_type, HolderEnum::Sheet);
    }

    #[test]
    fn from_parameter_uses_basic_parameter_values_when_present() {
        // 对应 Java：显式指定的 head/clazz 覆盖继承
        let parent = AbstractHolder::new(HolderEnum::Workbook);
        let mut parameter = BasicParameter::new();
        parameter.head = Some(vec![vec!["Age".to_owned()]]);
        parameter.clazz = Some("Other".to_owned());
        parameter.auto_trim = Some(false);
        parameter.use1904windowing = Some(true);
        parameter.locale = Some("zh-CN".to_owned());
        parameter.use_scientific_format = Some(true);
        parameter.filed_cache_location = Some(CacheLocation::ThreadLocal);

        let holder = AbstractHolder::from_parameter(&parameter, Some(&parent), HolderEnum::Sheet);
        assert_eq!(holder.head(), Some(&[vec!["Age".to_owned()]][..]));
        assert_eq!(holder.clazz(), Some("Other"));
        assert!(!holder.global_configuration.auto_trim);
        assert!(holder.global_configuration.use1904windowing);
        assert_eq!(holder.global_configuration.locale, "zh-CN");
        assert!(holder.global_configuration.use_scientific_format);
        assert_eq!(
            holder.global_configuration.filed_cache_location,
            CacheLocation::ThreadLocal
        );
    }

    #[test]
    fn accessors_return_none_when_unset() {
        // 对应 Java：未配置时 head/clazz 为空
        let holder = AbstractHolder::new(HolderEnum::Workbook);
        assert!(holder.head().is_none());
        assert!(holder.clazz().is_none());
    }
}

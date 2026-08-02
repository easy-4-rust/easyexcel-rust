//! Bridges [`GlobalConfiguration`] to [`ReadOptions`].

use easyexcel_core::metadata::GlobalConfiguration;

use crate::ReadOptions;
use crate::ScientificFormatMode;
use crate::locale::ExcelLocale;

/// Builds a global configuration snapshot from read options.
///
/// Mirrors Java holder propagation from `ReadBasicParameter` into
/// `GlobalConfiguration`.
#[must_use]
pub fn global_configuration_from_read_options(options: &ReadOptions) -> GlobalConfiguration {
    GlobalConfiguration {
        auto_trim: options.auto_trim,
        use1904windowing: options.use_1904_windowing,
        locale: options.locale.language_tag().to_owned(),
        use_scientific_format: matches!(
            options.scientific_format,
            ScientificFormatMode::Scientific
        ),
        filed_cache_location: easyexcel_core::CacheLocation::ThreadLocal,
    }
}

/// Applies a global configuration onto read options without replacing unrelated fields.
pub fn apply_global_configuration_to_read_options(
    global: &GlobalConfiguration,
    options: &mut ReadOptions,
) {
    options.auto_trim = global.auto_trim;
    options.use_1904_windowing = global.use1904windowing;
    if let Some(locale) = ExcelLocale::from_name(&global.locale) {
        options.locale = locale;
    }
    options.scientific_format = if global.use_scientific_format {
        ScientificFormatMode::Scientific
    } else {
        ScientificFormatMode::Plain
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn global_configuration_round_trips_read_options() {
        let mut options = ReadOptions::default();
        options.auto_trim = false;
        options.use_1904_windowing = true;
        options.scientific_format = ScientificFormatMode::Scientific;

        let global = global_configuration_from_read_options(&options);
        let mut restored = ReadOptions::default();
        apply_global_configuration_to_read_options(&global, &mut restored);

        assert_eq!(restored.auto_trim, options.auto_trim);
        assert_eq!(restored.use_1904_windowing, options.use_1904_windowing);
        assert_eq!(restored.scientific_format, options.scientific_format);
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;
    use easyexcel_core::metadata::GlobalConfiguration;

    #[test]
    fn apply_global_configuration_plain_mode_and_locale_name() {
        // 对应 Java：GlobalConfiguration 回填 ReadOptions 的 Plain 分支与 locale 名称
        let global = GlobalConfiguration {
            auto_trim: false,
            use1904windowing: true,
            locale: "en-US".to_owned(),
            use_scientific_format: false,
            filed_cache_location: easyexcel_core::CacheLocation::ThreadLocal,
        };
        let mut options = ReadOptions::default();
        options.scientific_format = ScientificFormatMode::Scientific;
        apply_global_configuration_to_read_options(&global, &mut options);
        assert_eq!(options.scientific_format, ScientificFormatMode::Plain);
        assert!(!options.auto_trim);
        assert!(options.use_1904_windowing);
        assert_eq!(options.locale.language_tag(), "en_US");
    }

    #[test]
    fn global_configuration_snapshot_reports_scientific_flag() {
        // 对应 Java：ReadBasicParameter -> GlobalConfiguration 快照
        let mut options = ReadOptions::default();
        options.scientific_format = ScientificFormatMode::Scientific;
        let global = global_configuration_from_read_options(&options);
        assert!(global.use_scientific_format);
        assert_eq!(global.locale, options.locale.language_tag());
        assert_eq!(global.auto_trim, options.auto_trim);
        assert_eq!(global.use1904windowing, options.use_1904_windowing);
    }
}

#[cfg(test)]
mod tests_extra2 {
    use super::*;

    #[test]
    fn global_configuration_snapshot_reports_plain_flag() {
        // 对应 Java：Plain 模式快照中 useScientificFormat=false
        let mut options = ReadOptions::default();
        options.scientific_format = ScientificFormatMode::Plain;
        let global = global_configuration_from_read_options(&options);
        assert!(!global.use_scientific_format);
    }
}

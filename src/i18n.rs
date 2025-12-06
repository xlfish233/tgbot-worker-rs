//! Lightweight internationalization (i18n) support for Telegram bots.
//!
//! This module provides a simple JSON-based translation system that is
//! compatible with wasm32 targets (Cloudflare Workers).
//!
//! # Example
//! ```ignore
//! use tgbot_worker_rs::i18n::I18n;
//!
//! let mut i18n = I18n::new("en");
//!
//! // Load translations from JSON
//! i18n.load_json("en", r#"{"hello": "Hello, {name}!", "bye": "Goodbye!"}"#)?;
//! i18n.load_json("zh", r#"{"hello": "你好，{name}！", "bye": "再见！"}"#)?;
//!
//! // Get translation
//! let text = i18n.t("en", "bye"); // "Goodbye!"
//!
//! // Get translation with arguments
//! let text = i18n.t_args("zh", "hello", &[("name", "世界")]); // "你好，世界！"
//! ```

use std::collections::HashMap;

/// A lightweight internationalization manager.
///
/// Stores translations as nested HashMaps: locale -> key -> value.
#[derive(Clone, Default)]
pub struct I18n {
    translations: HashMap<String, HashMap<String, String>>,
    default_locale: String,
}

impl I18n {
    /// Create a new I18n instance with a default locale.
    pub fn new(default_locale: &str) -> Self {
        Self {
            translations: HashMap::new(),
            default_locale: default_locale.to_string(),
        }
    }

    /// Load translations from a JSON string.
    ///
    /// The JSON should be a flat object: `{"key": "value", ...}`
    pub fn load_json(
        &mut self,
        locale: &str,
        json: &str,
    ) -> Result<(), serde_json::Error> {
        let map: HashMap<String, String> = serde_json::from_str(json)?;
        self.translations.insert(locale.to_string(), map);
        Ok(())
    }

    /// Add a single translation.
    pub fn add(&mut self, locale: &str, key: &str, value: &str) {
        self.translations
            .entry(locale.to_string())
            .or_default()
            .insert(key.to_string(), value.to_string());
    }

    /// Get a translation by key, falling back to default locale if not found.
    pub fn t<'a>(&'a self, locale: &str, key: &'a str) -> &'a str {
        // Try requested locale first
        if let Some(translations) = self.translations.get(locale)
            && let Some(value) = translations.get(key)
        {
            return value;
        }

        // Fall back to default locale
        if locale != self.default_locale
            && let Some(translations) = self.translations.get(&self.default_locale)
            && let Some(value) = translations.get(key)
        {
            return value;
        }

        // Return key as fallback
        key
    }

    /// Get a translation with argument substitution.
    ///
    /// Arguments in the translation string are formatted as `{name}`.
    ///
    /// # Example
    /// ```ignore
    /// // With translation: "Hello, {name}!"
    /// let text = i18n.t_args("en", "hello", &[("name", "World")]);
    /// // Result: "Hello, World!"
    /// ```
    pub fn t_args(&self, locale: &str, key: &str, args: &[(&str, &str)]) -> String {
        let mut text = self.t(locale, key).to_string();
        for (name, value) in args {
            text = text.replace(&format!("{{{}}}", name), value);
        }
        text
    }

    /// Get the default locale.
    pub fn default_locale(&self) -> &str {
        &self.default_locale
    }

    /// Set the default locale.
    pub fn set_default_locale(&mut self, locale: &str) {
        self.default_locale = locale.to_string();
    }

    /// Check if a locale has any translations.
    pub fn has_locale(&self, locale: &str) -> bool {
        self.translations.contains_key(locale)
    }

    /// Get all available locales.
    pub fn locales(&self) -> Vec<&str> {
        self.translations.keys().map(|s| s.as_str()).collect()
    }
}

/// Helper macro for inline translations.
///
/// # Example
/// ```ignore
/// let text = t!(i18n, "en", "hello");
/// let text = t!(i18n, "en", "hello", name = "World");
/// ```
#[macro_export]
macro_rules! t {
    ($i18n:expr, $locale:expr, $key:expr) => {
        $i18n.t($locale, $key)
    };
    ($i18n:expr, $locale:expr, $key:expr, $($name:ident = $val:expr),+ $(,)?) => {
        $i18n.t_args($locale, $key, &[$(( stringify!($name), $val )),+])
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_translation() {
        let mut i18n = I18n::new("en");
        i18n.add("en", "hello", "Hello!");
        i18n.add("zh", "hello", "你好！");

        assert_eq!(i18n.t("en", "hello"), "Hello!");
        assert_eq!(i18n.t("zh", "hello"), "你好！");
    }

    #[test]
    fn test_fallback_to_default() {
        let mut i18n = I18n::new("en");
        i18n.add("en", "hello", "Hello!");

        // zh doesn't have this key, should fall back to en
        assert_eq!(i18n.t("zh", "hello"), "Hello!");
    }

    #[test]
    fn test_fallback_to_key() {
        let i18n = I18n::new("en");

        // No translations, should return key
        assert_eq!(i18n.t("en", "missing"), "missing");
    }

    #[test]
    fn test_args_substitution() {
        let mut i18n = I18n::new("en");
        i18n.add("en", "greeting", "Hello, {name}! You have {count} messages.");

        let text = i18n.t_args("en", "greeting", &[("name", "Alice"), ("count", "5")]);
        assert_eq!(text, "Hello, Alice! You have 5 messages.");
    }

    #[test]
    fn test_load_json() {
        let mut i18n = I18n::new("en");
        i18n.load_json("en", r#"{"hello": "Hello!", "bye": "Goodbye!"}"#)
            .unwrap();

        assert_eq!(i18n.t("en", "hello"), "Hello!");
        assert_eq!(i18n.t("en", "bye"), "Goodbye!");
    }

    #[test]
    fn test_locales() {
        let mut i18n = I18n::new("en");
        i18n.add("en", "hello", "Hello!");
        i18n.add("zh", "hello", "你好！");
        i18n.add("ja", "hello", "こんにちは！");

        let locales = i18n.locales();
        assert_eq!(locales.len(), 3);
        assert!(locales.contains(&"en"));
        assert!(locales.contains(&"zh"));
        assert!(locales.contains(&"ja"));
    }
}

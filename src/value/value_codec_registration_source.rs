// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Value-codec registration source locations.

/// Source location for one linked value-codec registration.
///
/// The four parts are captured at compile time by the registration macros from
/// `CARGO_PKG_NAME`, `module_path!`, `file!`, and `line!`, so they point at the
/// declaration site rather than at the call site that later looks the
/// registration up. Ordering follows declaration order, which makes duplicate
/// reports deterministic.
///
/// # Examples
///
/// ```
/// use qubit_codec::ValueCodecRegistrationSource;
///
/// let source = ValueCodecRegistrationSource::new(
///     "qubit-codec",
///     "my_crate::codecs",
///     "src/codec.rs",
///     42,
/// );
/// assert_eq!(source.crate_name(), "qubit-codec");
/// assert_eq!(source.line(), 42);
/// ```
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ValueCodecRegistrationSource {
    /// Package name captured from `CARGO_PKG_NAME` at the declaration site.
    crate_name: &'static str,
    /// Module path captured from `module_path!` at the declaration site.
    module_path: &'static str,
    /// Source file captured from `file!` at the declaration site.
    file: &'static str,
    /// One-based line captured from `line!` at the declaration site.
    line: u32,
}

impl ValueCodecRegistrationSource {
    /// Creates a source location captured by a registration macro.
    ///
    /// # Parameters
    ///
    /// - `crate_name`: Package name captured from `CARGO_PKG_NAME`.
    /// - `module_path`: Module path captured from `module_path!`.
    /// - `file`: Source file captured from `file!`.
    /// - `line`: One-based line captured from `line!`.
    ///
    /// # Returns
    ///
    /// Returns a location that borrows all four captured parts statically.
    ///
    /// Prefer the registration macros over calling this directly.
    #[doc(hidden)]
    #[inline]
    #[must_use]
    pub const fn new(crate_name: &'static str, module_path: &'static str, file: &'static str, line: u32) -> Self {
        Self {
            crate_name,
            module_path,
            file,
            line,
        }
    }

    /// Returns the declaring crate name.
    ///
    /// # Returns
    ///
    /// Returns the `CARGO_PKG_NAME` of the crate that declared the codec.
    #[inline]
    #[must_use]
    pub const fn crate_name(&self) -> &'static str {
        self.crate_name
    }

    /// Returns the declaring module path.
    ///
    /// # Returns
    ///
    /// Returns the `module_path!` value of the declaring module.
    #[inline]
    #[must_use]
    pub const fn module_path(&self) -> &'static str {
        self.module_path
    }

    /// Returns the source file path.
    ///
    /// # Returns
    ///
    /// Returns the `file!` value of the declaring file.
    #[inline]
    #[must_use]
    pub const fn file(&self) -> &'static str {
        self.file
    }

    /// Returns the one-based source line.
    ///
    /// # Returns
    ///
    /// Returns the `line!` value of the declaring macro invocation.
    #[inline]
    #[must_use]
    pub const fn line(&self) -> u32 {
        self.line
    }
}

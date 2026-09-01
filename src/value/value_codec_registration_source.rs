// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Value-codec registration source locations.

/// Source location for one linked value-codec registration.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ValueCodecRegistrationSource {
    crate_name: &'static str,
    module_path: &'static str,
    file: &'static str,
    line: u32,
}

impl ValueCodecRegistrationSource {
    /// Creates a source location captured by a registration macro.
    #[doc(hidden)]
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
    #[must_use]
    pub const fn crate_name(&self) -> &'static str {
        self.crate_name
    }

    /// Returns the declaring module path.
    #[must_use]
    pub const fn module_path(&self) -> &'static str {
        self.module_path
    }

    /// Returns the source file path.
    #[must_use]
    pub const fn file(&self) -> &'static str {
        self.file
    }

    /// Returns the one-based source line.
    #[must_use]
    pub const fn line(&self) -> u32 {
        self.line
    }
}

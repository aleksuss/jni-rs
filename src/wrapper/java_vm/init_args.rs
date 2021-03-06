use std::{ffi::CString, os::raw::c_void};

use thiserror::Error;

use crate::{
    errors::Error as JniError,
    sys::{JavaVMInitArgs, JavaVMOption},
    JNIVersion,
};

/// Errors that can occur when invoking a [`JavaVM`](super::vm::JavaVM) with the
/// [Invocation API](https://docs.oracle.com/en/java/javase/12/docs/specs/jni/invocation.html).
#[derive(Debug, Error)]
pub enum JvmError {
    /// An internal `0` byte was found when constructing a string.
    #[error("internal null in option: {0}")]
    NullOptString(String),
}

impl From<JvmError> for JniError {
    fn from(e: JvmError) -> Self {
        JniError::JvmError(format!("{}", e))
    }
}

/// Builder for JavaVM InitArgs.
///
/// *This API requires "invocation" feature to be enabled,
/// see ["Launching JVM from Rust"](struct.JavaVM.html#launching-jvm-from-rust).*
#[derive(Debug)]
pub struct InitArgsBuilder {
    opts: Vec<String>,
    ignore_unrecognized: bool,
    version: JNIVersion,
}

impl Default for InitArgsBuilder {
    fn default() -> Self {
        InitArgsBuilder {
            opts: vec![],
            ignore_unrecognized: false,
            version: JNIVersion::V8,
        }
    }
}

impl InitArgsBuilder {
    /// Create a new default InitArgsBuilder
    pub fn new() -> Self {
        Default::default()
    }

    /// Add an option to the init args
    ///
    /// The `vfprintf`, `abort`, and `exit` options are unsupported at this time.
    pub fn option(self, opt_string: &str) -> Self {
        let mut s = self;

        match opt_string {
            "vfprintf" | "abort" | "exit" => return s,
            _ => {}
        }

        s.opts.push(opt_string.into());

        s
    }

    /// Set JNI version for the init args
    ///
    /// Default: V8
    pub fn version(self, version: JNIVersion) -> Self {
        let mut s = self;
        s.version = version;
        s
    }

    /// Set the `ignoreUnrecognized` init arg flag
    ///
    /// If ignoreUnrecognized is true, JavaVM::new ignores all unrecognized option strings that
    /// begin with "-X" or "_". If ignoreUnrecognized is false, JavaVM::new returns Err as soon as
    /// it encounters any unrecognized option strings.
    ///
    /// Default: `false`
    pub fn ignore_unrecognized(self, ignore: bool) -> Self {
        let mut s = self;
        s.ignore_unrecognized = ignore;
        s
    }

    /// Build the `InitArgs`
    ///
    /// This will check for internal nulls in the option strings and will return
    /// an error if one is found.
    pub fn build(self) -> Result<InitArgs, JvmError> {
        let mut opts = Vec::with_capacity(self.opts.len());
        for opt in self.opts {
            let option_string =
                CString::new(opt.as_str()).map_err(|_| JvmError::NullOptString(opt))?;
            let jvm_opt = JavaVMOption {
                optionString: option_string.into_raw(),
                extraInfo: ::std::ptr::null_mut(),
            };
            opts.push(jvm_opt);
        }

        Ok(InitArgs {
            inner: JavaVMInitArgs {
                version: self.version.into(),
                ignoreUnrecognized: self.ignore_unrecognized as _,
                options: opts.as_ptr() as _,
                nOptions: opts.len() as _,
            },
            opts,
        })
    }

    /// Returns collected options
    pub fn options(&self) -> Vec<String> {
        self.opts.clone()
    }
}

/// JavaVM InitArgs.
///
/// *This API requires "invocation" feature to be enabled,
/// see ["Launching JVM from Rust"](struct.JavaVM.html#launching-jvm-from-rust).*
pub struct InitArgs {
    inner: JavaVMInitArgs,
    opts: Vec<JavaVMOption>,
}

impl InitArgs {
    pub(crate) fn inner_ptr(&self) -> *mut c_void {
        &self.inner as *const _ as _
    }
}

impl Drop for InitArgs {
    fn drop(&mut self) {
        for opt in self.opts.iter() {
            unsafe { CString::from_raw(opt.optionString) };
        }
    }
}

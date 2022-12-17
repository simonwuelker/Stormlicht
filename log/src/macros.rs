#[macro_export]
/// Note that this function does not respect the log level.
/// Prefer to use the level-specific log macros instead.
macro_rules! log {
    (target = $target:expr, $level: expr, $($arg:tt)+) => {
        let log_message = log::LogMessage {
            source: (std::file!(), std::column!()),
            level: $level,
            target: Some($target),
            text: std::format!($($arg)+),
        };
        log::log_generic(log_message);
    };
    ($level: expr, $($arg:tt)+) => {
        let log_message = log::LogMessage {
            source: (std::file!(), std::column!()),
            level: $level,
            target: None,
            text: std::format!($($arg)+),
        };
        log::log_generic(log_message);

    };
}

#[cfg(feature = "log-error")]
#[macro_export]
macro_rules! error {
    (target = $target:expr, $($arg:tt)+) => (log::log!(target = $target, log::Level::Error, $($arg)+));
    ($($arg:tt)+) => (log::log!(log::Level::Error, $($arg)+));
}

#[cfg(not(feature = "log-error"))]
#[macro_export]
macro_rules! error {
    (target = $target:expr, $($arg:tt)+) => {};
    ($($arg:tt)+) => {};
}

#[cfg(feature = "log-warn")]
#[macro_export]
macro_rules! warn {
    (target = $target:expr, $($arg:tt)+) => (log::log!(target = $target, log::Level::Warn, $($arg)+));
    ($($arg:tt)+) => (log::log!(log::Level::Warn, $($arg)+))

}

#[cfg(not(feature = "log-warn"))]
#[macro_export]
macro_rules! warn {
    (target = $target:expr, $($arg:tt)+) => {};
    ($($arg:tt)+) => {};
}

#[cfg(feature = "log-info")]
#[macro_export]
macro_rules! info {
    (target = $target:expr, $($arg:tt)+) => (log::log!(target = $target, log::Level::Warn, $($arg)+));
    ($($arg:tt)+) => (log::log!(log::Level::Info, $($arg)+))
}

#[cfg(not(feature = "log-info"))]
#[macro_export]
macro_rules! info {
    (target = $target:expr, $($arg:tt)+) => {};
    ($($arg:tt)+) => {};
}

#[cfg(feature = "log-debug")]
#[macro_export]
macro_rules! debug {
    (target = $target:expr, $($arg:tt)+) => (log::log!(target = $target, log::Level::Warn, $($arg)+));
    ($($arg:tt)+) => (log::log!(log::Level::Debug, $($arg)+))
}

#[cfg(not(feature = "log-debug"))]
#[macro_export]
macro_rules! debug {
    (target = $target:expr, $($arg:tt)+) => {};
    ($($arg:tt)+) => {};
}

#[cfg(feature = "log-trace")]
#[macro_export]
macro_rules! trace {
    (target = $target:expr, $($arg:tt)+) => (log::log!(target = $target, log::Level::Warn, $($arg)+));
    ($($arg:tt)+) => (log::log!(log::Level::Trace, $($arg)+))
}

#[cfg(not(feature = "log-trace"))]
#[macro_export]
macro_rules! trace {
    (target = $target:expr, $($arg:tt)+) => {};
    ($($arg:tt)+) => {};
}

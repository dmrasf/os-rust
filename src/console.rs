#![allow(unused)]

use crate::sbi::console_putchar;
use core::fmt::{self, Write};

struct Stdout;

#[derive(PartialEq, PartialOrd)]
pub enum LOG {
    DISABLE = -1,
    ERROR = 0,
    WARN = 1,
    INFO = 2,
    DEBUG = 3,
    TRACE = 4,
}

pub const LOG_LEVEL: LOG = LOG::TRACE;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            console_putchar(c as usize);
        }
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! error {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        if $crate::console::LOG_LEVEL >= $crate::console::LOG::ERROR {
            $crate::console::print(format_args!(concat!("\x1b[1;31m[ERROR]\x1b[0m\x1b[31m ", $fmt, "\x1b[0m\n") $(, $($arg)+)?));
        }
    }
}

#[macro_export]
macro_rules! warn {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        if $crate::console::LOG_LEVEL >= $crate::console::LOG::WARN {
            $crate::console::print(format_args!(concat!("\x1b[1;93m[WARN]\x1b[0m\x1b[93m ", $fmt, "\x1b[0m\n") $(, $($arg)+)?));
        }
    }
}
#[macro_export]
macro_rules! info {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        if $crate::console::LOG_LEVEL >= $crate::console::LOG::INFO {
            $crate::console::print(format_args!(concat!("\x1b[1;34m[INFO]\x1b[0m\x1b[34m ", $fmt, "\x1b[0m\n") $(, $($arg)+)?));
        }
    }
}
#[macro_export]
macro_rules! debug {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        if $crate::console::LOG_LEVEL >= $crate::console::LOG::DEBUG {
            $crate::console::print(format_args!(concat!("\x1b[1;32m[DEBUG]\x1b[0m\x1b[32m ", $fmt, "\x1b[0m\n") $(, $($arg)+)?));
        }
    }
}
#[macro_export]
macro_rules! trace {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        if $crate::console::LOG_LEVEL >= $crate::console::LOG::TRACE {
            $crate::console::print(format_args!(concat!("\x1b[1;90m[TRACE]\x1b[0m\x1b[90m ", $fmt, "\x1b[0m\n") $(, $($arg)+)?));
        }
    }
}

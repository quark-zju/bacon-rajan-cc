// Copyright 2015 The Rust Project Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT
// or http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use cc_box_ptr::CcBoxPtr;

/// A `Tracer` is a callback function that is invoked for each `CcBoxPtr` owned
/// by an instance of something.
pub type Tracer = FnMut(&CcBoxPtr);

/// A trait that informs cycle collector how to find memory that is owned by a
/// `Trace` instance and managed by the cycle collector.
pub trait Trace {
    /// Invoke the `Tracer` on each of the `CcBoxPtr`s owned by this `Trace`
    /// instance.
    ///
    /// Failing to invoke the tracer on every owned `CcBoxPtr` can lead to
    /// leaking cycles.
    ///
    /// This function will be skipped if `is_atomic` returns `true`.
    fn trace(&self, tracer: &mut Tracer);

    /// Returning true to opt-out being tracked by the cycle collector, and
    /// behave more like an `Rc`.
    fn is_atomic(&self) -> bool { false }
}

#[inline]
pub(crate) fn trace_non_atomic(this: &CcBoxPtr, tracer: &mut Tracer) {
    if !this.is_atomic() {
        this.trace(tracer);
    }
}

#[macro_export]
/// Mark types as atomic. Atomic types opt-out the cycle collector.
macro_rules! atomic {
    ( $( $t: ty ),* ) => {
        $(
            impl $crate::Trace for $t {
                fn trace(&self, _tracer: &mut $crate::Tracer) {}
                fn is_atomic(&self) -> bool { true }
            }
        )*
    }
}

mod impls {
    pub use super::*;

    mod primitives {
        pub use super::*;

        atomic!(bool, char, f32, f64, i16, i32, i64, i8, isize, str, u16, u32, u64, u8, usize);

        impl<'a, T: Trace> Trace for &'a mut [T] {
            fn trace(&self, tracer: &mut Tracer) {
                for t in &self[..] {
                    t.trace(tracer);
                }
            }
        }

        mod arrays {
            pub use super::*;

            // impl<T: Trace> Trace for [T; 0] {
            // }
            // impl<T: Trace> Trace for [T; 1] {
            // }
            // impl<T: Trace> Trace for [T; 2] {
            // }
            // impl<T: Trace> Trace for [T; 3] {
            // }
            // impl<T: Trace> Trace for [T; 4] {
            // }
            // impl<T: Trace> Trace for [T; 5] {
            // }
            // impl<T: Trace> Trace for [T; 6] {
            // }
            // impl<T: Trace> Trace for [T; 7] {
            // }
            // impl<T: Trace> Trace for [T; 8] {
            // }
            // impl<T: Trace> Trace for [T; 9] {
            // }
            // impl<T: Trace> Trace for [T; 10] {
            // }
            // impl<T: Trace> Trace for [T; 11] {
            // }
            // impl<T: Trace> Trace for [T; 12] {
            // }
            // impl<T: Trace> Trace for [T; 13] {
            // }
            // impl<T: Trace> Trace for [T; 14] {
            // }
            // impl<T: Trace> Trace for [T; 15] {
            // }
            // impl<T: Trace> Trace for [T; 16] {
            // }
            // impl<T: Trace> Trace for [T; 17] {
            // }
            // impl<T: Trace> Trace for [T; 18] {
            // }
            // impl<T: Trace> Trace for [T; 19] {
            // }
            // impl<T: Trace> Trace for [T; 20] {
            // }
            // impl<T: Trace> Trace for [T; 21] {
            // }
            // impl<T: Trace> Trace for [T; 22] {
            // }
            // impl<T: Trace> Trace for [T; 23] {
            // }
            // impl<T: Trace> Trace for [T; 24] {
            // }
            // impl<T: Trace> Trace for [T; 25] {
            // }
            // impl<T: Trace> Trace for [T; 26] {
            // }
            // impl<T: Trace> Trace for [T; 27] {
            // }
            // impl<T: Trace> Trace for [T; 28] {
            // }
            // impl<T: Trace> Trace for [T; 29] {
            // }
            // impl<T: Trace> Trace for [T; 30] {
            // }
            // impl<T: Trace> Trace for [T; 31] {
            // }
            // impl<T: Trace> Trace for [T; 32] {
            // }
        }

        mod tuples {
            atomic!(());
            // impl Trace for tuple {
            // }
        }
    }

    mod boxed {
        pub use super::*;

        impl<T: Trace + ?Sized> Trace for Box<T> {
            fn trace(&self, tracer: &mut Tracer) {
                (**self).trace(tracer);
            }
        }
    }

    mod cell {
        pub use super::*;
        use std::cell;

        impl<T: Copy + Trace + ?Sized> Trace for cell::Cell<T> {
            fn trace(&self, tracer: &mut Tracer) {
                self.get().trace(tracer);
            }
        }

        impl<T: Trace + ?Sized> Trace for cell::RefCell<T> {
            fn trace(&self, tracer: &mut Tracer) {
                // If the RefCell is currently borrowed we
                // assume there's an outstanding reference to this
                // cycle so it's ok if we don't trace through it.
                // If the borrow gets leaked somehow then we're going
                // to leak the cycle.
                if let Ok(x) = self.try_borrow() {
                    x.trace(tracer);
                }
            }
        }
    }

    mod collections {
        pub use super::*;
        use std::collections;
        use std::hash;

        impl<K, V: Trace> Trace for collections::BTreeMap<K, V> {
            fn trace(&self, tracer: &mut Tracer) {
                for (_, v) in self {
                    v.trace(tracer);
                }
            }
        }

        impl<K: Eq + hash::Hash + Trace, V: Trace> Trace for collections::HashMap<K, V> {
            fn trace(&self, tracer: &mut Tracer) {
                for (_, v) in self {
                    v.trace(tracer);
                }
            }
        }

        impl<T: Trace> Trace for collections::LinkedList<T> {
            fn trace(&self, tracer: &mut Tracer) {
                for t in self {
                    t.trace(tracer);
                }
            }
        }

        impl<T: Trace> Trace for collections::VecDeque<T> {
            fn trace(&self, tracer: &mut Tracer) {
                for t in self {
                    t.trace(tracer);
                }
            }
        }
    }

    mod vec {
        pub use super::*;
        impl<T: Trace> Trace for Vec<T> {
            fn trace(&self, tracer: &mut Tracer) {
                for t in self {
                    t.trace(tracer);
                }
            }
        }
    }

    mod string {
        pub use super::*;

        atomic!(String);
    }

    mod func {
        pub use super::*;

        impl<X> Trace for fn() -> X {
            fn trace(&self, _tracer: &mut Tracer) { }
            fn is_atomic(&self) -> bool { true }
        }

        impl<A, X> Trace for fn(A) -> X {
            fn trace(&self, _tracer: &mut Tracer) { }
            fn is_atomic(&self) -> bool { true }
        }

        impl<A, B, X> Trace for fn(A, B) -> X {
            fn trace(&self, _tracer: &mut Tracer) { }
            fn is_atomic(&self) -> bool { true }
        }

        impl<A, B, C, X> Trace for fn(A, B, C) -> X {
            fn trace(&self, _tracer: &mut Tracer) { }
            fn is_atomic(&self) -> bool { true }
        }

        impl<A, B, C, D, X> Trace for fn(A, B, C, D) -> X {
            fn trace(&self, _tracer: &mut Tracer) { }
            fn is_atomic(&self) -> bool { true }
        }

        impl<A, B, C, D, E, X> Trace for fn(A, B, C, D, E) -> X {
            fn trace(&self, _tracer: &mut Tracer) { }
            fn is_atomic(&self) -> bool { true }
        }

        impl<A, B, C, D, E, F, X> Trace for fn(A, B, C, D, E, F) -> X {
            fn trace(&self, _tracer: &mut Tracer) { }
            fn is_atomic(&self) -> bool { true }
        }

        impl<A, B, C, D, E, F, G, X> Trace for fn(A, B, C, D, E, F, G) -> X {
            fn trace(&self, _tracer: &mut Tracer) { }
            fn is_atomic(&self) -> bool { true }
        }
    }

    mod ffi {
        pub use super::*;
        use std::ffi;

        atomic!(ffi::CStr, ffi::CString, ffi::NulError, ffi::OsStr, ffi::OsString);
    }

    mod io {
        pub use super::*;
        use std::io;

        impl<T> Trace for io::BufReader<T> {
            fn trace(&self, _tracer: &mut Tracer) { }
            fn is_atomic(&self) -> bool { true }
        }

        impl<T: io::Write> Trace for io::BufWriter<T> {
            fn trace(&self, _tracer: &mut Tracer) { }
            fn is_atomic(&self) -> bool { true }
        }

        impl<T> Trace for io::Cursor<T> {
            fn trace(&self, _tracer: &mut Tracer) { }
            fn is_atomic(&self) -> bool { true }
        }

        impl Trace for io::Empty {
            fn trace(&self, _tracer: &mut Tracer) { }
            fn is_atomic(&self) -> bool { true }
        }

        impl Trace for io::Error {
            fn trace(&self, _tracer: &mut Tracer) { }
            fn is_atomic(&self) -> bool { true }
        }

        impl<T> Trace for io::IntoInnerError<T> {
            fn trace(&self, _tracer: &mut Tracer) { }
            fn is_atomic(&self) -> bool { true }
        }

        impl<T: io::Write> Trace for io::LineWriter<T> {
            fn trace(&self, _tracer: &mut Tracer) { }
            fn is_atomic(&self) -> bool { true }
        }

        impl<T> Trace for io::Lines<T> {
            fn trace(&self, _tracer: &mut Tracer) { }
            fn is_atomic(&self) -> bool { true }
        }

        impl Trace for io::Repeat {
            fn trace(&self, _tracer: &mut Tracer) { }
            fn is_atomic(&self) -> bool { true }
        }

        impl Trace for io::Sink {
            fn trace(&self, _tracer: &mut Tracer) { }
            fn is_atomic(&self) -> bool { true }
        }

        impl<T> Trace for io::Split<T> {
            fn trace(&self, _tracer: &mut Tracer) { }
            fn is_atomic(&self) -> bool { true }
        }

        impl Trace for io::Stderr {
            fn trace(&self, _tracer: &mut Tracer) { }
            fn is_atomic(&self) -> bool { true }
        }

        impl Trace for io::Stdin {
            fn trace(&self, _tracer: &mut Tracer) { }
            fn is_atomic(&self) -> bool { true }
        }

        impl Trace for io::Stdout {
            fn trace(&self, _tracer: &mut Tracer) { }
            fn is_atomic(&self) -> bool { true }
        }

        impl<T> Trace for io::Take<T> {
            fn trace(&self, _tracer: &mut Tracer) { }
            fn is_atomic(&self) -> bool { true }
        }
    }

    mod net {
        pub use super::*;
        use std::net;

        atomic!(
            net::AddrParseError,
            net::Ipv4Addr,
            net::Ipv6Addr,
            net::SocketAddrV4,
            net::SocketAddrV6,
            net::TcpListener,
            net::TcpStream,
            net::UdpSocket
        );
    }

    mod option {
        pub use super::*;

        impl<T: Trace> Trace for Option<T> {
            fn trace(&self, tracer: &mut Tracer) {
                if let Some(ref t) = *self {
                    t.trace(tracer);
                }
            }
        }
    }

    mod path {
        pub use super::*;
        use std::path;

        atomic!(path::Path, path::PathBuf);
    }

    mod process {
        pub use super::*;
        use std::process;

        atomic!(
            process::Child,
            process::ChildStderr,
            process::ChildStdin,
            process::ChildStdout,
            process::Command,
            process::ExitStatus,
            process::Output,
            process::Stdio
        );
    }

    mod rc {
        pub use super::*;
        use std::rc;

        impl<T> Trace for rc::Rc<T> {
            fn trace(&self, _tracer: &mut Tracer) { }
            fn is_atomic(&self) -> bool { true }
        }

        impl<T> Trace for rc::Weak<T> {
            fn trace(&self, _tracer: &mut Tracer) { }
            fn is_atomic(&self) -> bool { true }
        }
    }

    mod result {
        pub use super::*;

        impl<T: Trace, U: Trace> Trace for Result<T, U> {
            fn trace(&self, tracer: &mut Tracer) {
                match *self {
                    Ok(ref t) => t.trace(tracer),
                    Err(ref u) => u.trace(tracer),
                }
            }
        }
    }

    mod sync {
        pub use super::*;
        use std::sync;

        impl<T> Trace for sync::Arc<T> {
            fn trace(&self, _tracer: &mut Tracer) { }
            fn is_atomic(&self) -> bool { true }
        }

        impl Trace for sync::Barrier {
            fn trace(&self, _tracer: &mut Tracer) { }
            fn is_atomic(&self) -> bool { true }
        }

        impl Trace for sync::Condvar {
            fn trace(&self, _tracer: &mut Tracer) { }
            fn is_atomic(&self) -> bool { true }
        }

        impl<T> Trace for sync::Mutex<T> {
            fn trace(&self, _tracer: &mut Tracer) { }
            fn is_atomic(&self) -> bool { true }
        }

        impl Trace for sync::Once {
            fn trace(&self, _tracer: &mut Tracer) { }
            fn is_atomic(&self) -> bool { true }
        }

        impl<T> Trace for sync::PoisonError<T> {
            fn trace(&self, _tracer: &mut Tracer) { }
            fn is_atomic(&self) -> bool { true }
        }

        impl<T: Trace> Trace for sync::RwLock<T> {
            fn trace(&self, tracer: &mut Tracer) {
                if let Ok(v) = self.write() {
                    v.trace(tracer);
                }
            }
        }
    }

    mod thread {
        pub use super::*;
        use std::thread;

        impl Trace for thread::Builder {
            fn trace(&self, _tracer: &mut Tracer) { }
            fn is_atomic(&self) -> bool { true }
        }

        impl<T> Trace for thread::JoinHandle<T> {
            fn trace(&self, _tracer: &mut Tracer) { }
            fn is_atomic(&self) -> bool { true }
        }

        impl<T> Trace for thread::LocalKey<T> {
            fn trace(&self, _tracer: &mut Tracer) { }
            fn is_atomic(&self) -> bool { true }
        }

        impl Trace for thread::Thread {
            fn trace(&self, _tracer: &mut Tracer) { }
            fn is_atomic(&self) -> bool { true }
        }
    }
}

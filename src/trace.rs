//          Copyright Nick G 2021.
// Distributed under the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE or copy at
//          https://www.boost.org/LICENSE_1_0.txt)

use std::time::Duration;

/// represents a trace object
#[derive(PartialEq, Debug)]
pub struct Trace {
    pub name: String,
    pub start: Duration,
    pub duration: Duration,
    pub thread_number: u32,
}

impl Serialize for Trace {
    /// In order to serialize and meet the tracing format,
    /// https://docs.google.com/document/d/1CvAClvFfyA5R-PhYUmn5OOQtYMH4h6I0nSsKchNAySU/preview#heading=h.f2f0yd51wi15,
    /// we need:
    ///
    ///     {
    ///        "name": "string",
    ///        "cat": "string",
    ///        "ph": "X",
    ///        "ts": int,
    ///        "dur": int,
    ///        "pid": int,
    ///        "tid": int,
    ///        "args": {}
    ///    }
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        serializer.serialize_i32(*self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_first_test() {
        let trace = Trace{name: "foo".into(), start: Duration::from_millis(0), duration: Duration::from_millis(300), thread_number: 2};
        
    }
}

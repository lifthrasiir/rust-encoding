// This is a part of rust-encoding.
// Copyright (c) 2013, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

//! Macros for testing.

#[macro_escape];

macro_rules! assert_feed_ok(
    ($this:expr, $processed:expr, $unprocessed:expr, $output:expr) => ({
        let processed = $this.test_norm_input(&$processed);
        let unprocessed = $this.test_norm_input(&$unprocessed);
        let output = $this.test_norm_output(&$output);
        let input = processed + unprocessed;
        let (nprocessed, err, buf) = $this.test_feed(input);
        let upto = err.map(|e| e.upto);
        assert!(processed.len() == nprocessed && None == upto,
                "raw_feed should return %?, but instead returned %?",
                (processed.len(), None::<uint>), (nprocessed, upto));
        assert!(output == buf.as_slice(),
                "raw_feed should push %?, but instead pushed %?", output, buf.as_slice());
    })
)

macro_rules! assert_feed_err(
    ($this:expr, $processed:expr, $problem:expr, $remaining:expr, $output:expr) => ({
        let processed = $this.test_norm_input(&$processed);
        let problem = $this.test_norm_input(&$problem);
        let remaining = $this.test_norm_input(&$remaining);
        let output = $this.test_norm_output(&$output);
        let input = processed + problem + remaining;
        let (nprocessed, err, buf) = $this.test_feed(input);
        let upto = err.map(|e| e.upto);
        assert!(processed.len() == nprocessed && Some(processed.len() + problem.len()) == upto,
                "raw_feed should return %?, but instead returned %?",
                (processed.len(), Some(processed.len() + problem.len())), (nprocessed, upto));
        assert!(output == buf.as_slice(),
                "raw_feed should push %?, but instead pushed %?", output, buf.as_slice());
    })
)

macro_rules! assert_finish_ok(
    ($this:expr, $output:expr) => ({
        let output = $this.test_norm_output(&$output);
        let (err, buf) = $this.test_finish();
        let upto = err.map(|e| e.upto);
        assert!(None == upto,
                "raw_finish should return %?, but instead returned %?", None::<uint>, upto);
        assert!(output == buf.as_slice(),
                "raw_finish should push %?, but instead pushed %?", output, buf.as_slice());
    })
)

macro_rules! assert_finish_err(
    ($this:expr, $output:expr) => ({
        let output = $this.test_norm_output(&$output);
        let (err, buf) = $this.test_finish();
        let upto = err.map(|e| e.upto);
        assert!(Some(0) == upto,
                "raw_finish should return %?, but instead returned %?", Some(0), upto);
        assert!(output == buf.as_slice(),
                "raw_finish should push %?, but instead pushed %?", output, buf.as_slice());
    })
)


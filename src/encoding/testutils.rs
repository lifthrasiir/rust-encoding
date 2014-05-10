// This is a part of rust-encoding.
// Copyright (c) 2013, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

//! Macros and utilities for testing.

#![macro_escape]

macro_rules! assert_feed_ok(
    ($this:expr, $processed:expr, $unprocessed:expr, $output:expr) => ({
        let processed = $processed;
        let processed = $this.test_norm_input(processed);
        let unprocessed = $unprocessed;
        let unprocessed = $this.test_norm_input(unprocessed);
        let output = $output;
        let output = $this.test_norm_output(output);
        let input = $this.test_concat(processed, unprocessed);
        let (nprocessed, err, buf) = $this.test_feed(input.as_slice());
        let upto = err.map(|e| e.upto);
        assert!(processed.len() == nprocessed && None == upto,
                "raw_feed should return {:?}, but instead returned {:?}",
                (processed.len(), None::<uint>), (nprocessed, upto));
        assert!(output == buf.as_slice(),
                "raw_feed should push {:?}, but instead pushed {:?}", output, buf.as_slice());
    })
)

macro_rules! assert_feed_err(
    ($this:expr, $processed:expr, $problem:expr, $remaining:expr, $output:expr) => ({
        let processed = $processed;
        let processed = $this.test_norm_input(processed);
        let problem = $problem;
        let problem = $this.test_norm_input(problem);
        let remaining = $remaining;
        let remaining = $this.test_norm_input(remaining);
        let output = $output;
        let output = $this.test_norm_output(output);
        let input = $this.test_concat($this.test_concat(processed, problem).as_slice(), remaining);
        let (nprocessed, err, buf) = $this.test_feed(input.as_slice());
        let upto = err.map(|e| e.upto);
        assert!(processed.len() == nprocessed && Some(processed.len() + problem.len()) == upto,
                "raw_feed should return {:?}, but instead returned {:?}",
                (processed.len(), Some(processed.len() + problem.len())), (nprocessed, upto));
        assert!(output == buf.as_slice(),
                "raw_feed should push {:?}, but instead pushed {:?}", output, buf.as_slice());
    })
)

macro_rules! assert_finish_ok(
    ($this:expr, $output:expr) => ({
        let output = $output;
        let output = $this.test_norm_output(output);
        let (err, buf) = $this.test_finish();
        let upto = err.map(|e| e.upto);
        assert!(None == upto,
                "raw_finish should return {:?}, but instead returned {:?}", None::<uint>, upto);
        assert!(output == buf.as_slice(),
                "raw_finish should push {:?}, but instead pushed {:?}", output, buf.as_slice());
    })
)

macro_rules! assert_finish_err(
    ($this:expr, $output:expr) => ({
        let output = $output;
        let output = $this.test_norm_output(output);
        let (err, buf) = $this.test_finish();
        let upto = err.map(|e| e.upto);
        assert!(Some(0) == upto,
                "raw_finish should return {:?}, but instead returned {:?}", Some(0), upto);
        assert!(output == buf.as_slice(),
                "raw_finish should push {:?}, but instead pushed {:?}", output, buf.as_slice());
    })
)

/// Some Korean text to test.
//
// the first paragraphs of the article "Korean Language" from Korean Wikipedia.
// https://ko.wikipedia.org/w/index.php?title=%ED%95%9C%EA%B5%AD%EC%96%B4&oldid=12331875
pub static KOREAN_TEXT: &'static str =
    "한국어(韓國語)는 주로 한반도(韓半島)와 한민족(韓民族) 거주 지역에서 쓰이는 언어로, \
     대한민국에서는 한국어, 한국말이라고 부르고, 조선민주주의인민공화국과 중국, 일본에서는 \
     조선어(朝鮮語), 조선말이라고 불린다. 우즈베키스탄, 러시아 등 구 소련의 고려인들 사이에서는 \
     고려말(高麗語)로 불린다. 19세기 중반 이후 한반도와 주변 정세의 혼란, 20세기 전반 \
     일본 제국주의의 침략, 20세기 후반 대한민국의 해외 이민에 의해 중국 동북 지방, 일본, \
     러시아 연해주와 사할린, 우즈베키스탄, 미국, 캐나다, 오스트레일리아, 필리핀, 베트남, 브라질 등 \
     세계 곳곳에 한민족이 이주하면서 한국어가 쓰이고 있다. 한국어 쓰는 인구는 전 세계를 통틀어 \
     약 8천250만 명으로 추산된다.";

/// Some Japanese text to test.
//
// the first paragraphs of the article "Japanese Language" from Japanese Wikipedia.
// https://ja.wikipedia.org/w/index.php?title=%E6%97%A5%E6%9C%AC%E8%AA%9E&oldid=51443986
pub static JAPANESE_TEXT: &'static str =
    "日本語（にほんご、にっぽんご）とは、主に日本国内や日本人同士の間で使われている言語である。\
     日本は法令によって「公用語」を規定していないが、法令その他の公用文は日本語で記述され、\
     各種法令（裁判所法第74条、会社計算規則第57条、特許法施行規則第2条など）において\
     日本語を用いることが定められるなど事実上の公用語となっており、学校教育の「国語」でも\
     教えられる。使用人口について正確な統計はないが、日本国内の人口、および日本国外に住む\
     日本人や日系人、日本がかつて統治した地域の一部の住民など、約1億3千万人以上と考えられる。\
     統計によって前後する可能性はあるが、この数は世界の母語話者数で上位10位以内に入る人数である。";


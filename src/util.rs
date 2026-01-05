use std::ops::Range;
use std::slice;
use std::str::from_utf8_unchecked;

unsafe fn range_to_str<'a>(range: Range<*const u8>) -> &'a str {
    let Range { start, end } = range;
    unsafe {
        from_utf8_unchecked(slice::from_raw_parts(
            start,
            end.offset_from(start) as usize,
        ))
    }
}

#[allow(dead_code)]
pub trait TextUtility {
    fn between<'a>(self: &'a Self, start: &str, end: &str) -> Option<(&'a str, &'a str)>;
    fn skip_until<'a>(self: &'a Self, t: &str) -> Option<&'a str>;
    fn skip_while<'a>(self: &'a Self, p: impl Fn(char) -> bool) -> &'a str;
}

impl TextUtility for str {
    fn between<'a>(self: &'a Self, start: &str, end: &str) -> Option<(&'a str, &'a str)> {
        let target_start = self.matches(start).next()?.as_bytes().as_ptr_range().end;
        let base_end = self.as_bytes().as_ptr_range().end;
        let target = unsafe { range_to_str::<'a>(target_start..base_end) };
        let Range {
            start: target_end,
            end: rest_start,
        } = target.matches(end).next()?.as_bytes().as_ptr_range();
        Some((
            unsafe { range_to_str::<'a>(target_start..target_end) },
            unsafe { range_to_str::<'a>(rest_start..base_end) },
        ))
    }

    fn skip_until<'a>(self: &'a str, t: &str) -> Option<&'a str> {
        let range_start = self.matches(t).next()?.as_bytes().as_ptr_range().end;
        let range_end = self.as_bytes().as_ptr_range().end;
        Some(unsafe { range_to_str::<'a>(range_start..range_end) })
    }

    fn skip_while<'a>(self: &'a str, p: impl Fn(char) -> bool) -> &'a str {
        let mut iter = self.chars();
        while let Some(c) = iter.next() {
            if !p(c) {
                break;
            }
        }
        iter.as_str()
    }
}

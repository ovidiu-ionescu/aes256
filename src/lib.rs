mod aes;

use std::str::{CharIndices};

struct  Part <'a> {
    text: &'a str,
    clear: bool,
}

struct Status {
    char_count: usize,
    equal_count: usize,
    inside: bool,
    found: isize,
    prev_pos: usize,
    parts: Vec<(usize, usize, bool)>,
}

impl Status {
    fn new () -> Status {
        Status {
            char_count: 0,
            equal_count: 0,
            // wether we think we are inside a b64 string
            inside: false,
            found: 0,
            // the last position that was included in the parts
            prev_pos: 0,
            parts: Vec::with_capacity(10),
        }
    }

    /// Check if we have advanced to the end of a base64 segment
    fn check_base64(&mut self, pos: usize) {
        let total_chars = self.char_count + self.equal_count;
        if total_chars >= 16 && total_chars %4 == 0 && self.inside {
            self.found += 1;
            // add the clear string if it is non empty
            if self.prev_pos < pos - total_chars {
                self.parts.push((self.prev_pos, pos - total_chars, false));
            }
            self.parts.push((pos - total_chars, pos, true));
            println!("Found a base64");
            self.prev_pos = pos;
        }
        // println!("prev_pos: {}, total_chars {}", self.prev_pos, total_chars);
        self.char_count = 0;
        self.equal_count = 0;
        self.inside = false;
    }
}

pub fn b64here(s: &str) -> Vec<(usize, usize, bool)> {
    let mut status = Status::new();

    for (pos, c) in s.char_indices() {
        print!("{}", c);
        if status.inside {
            // we can only have = signs at the end and max three of them
            if (c != '=' && status.equal_count > 0) || (c == '=' && status.equal_count > 2) {
                status.check_base64(pos);
                continue;
            }

            if c == '=' {
                status.equal_count += 1;
                continue;
            }
        }

        if ('A'..='Z').contains(&c) || ('a'..= 'z').contains(&c) || ('0' ..= '9').contains(&c) || '/' == c || '+' == c {
            status.char_count += 1;
            status.inside = true;
        } else {
            if status.inside  {
                status.check_base64(pos);
            }
        }
    }
    status.check_base64(s.len());
    if status.prev_pos < s.len() {
        status.parts.push((status.prev_pos, s.len(), false));
    }
    status.parts
 }

#[cfg(test)]
mod tests {

    #[test]
    fn b64here_test() {
        let s = r#"
This is a sample test text

First secret
8xRyXaSkpKQGqlTMpMssgnNsZDnatopg

Second secret
/xRyXY6Ojo7/u45hZut8f41Uf6C2GvNCdA==

Third secret
DhVyXUxMTEwpo9eX4aw7dJnT1zaZ9DBqISbEU0rj6pPcoWZk5m1xTDqQouV4pyOxdLLVIeBfZG/bF2Rlm4AVR7dnn28t8Sr5
Fourth secret
JhVyXYWFhYVkprM94+hLMA=="#;
        let parts = super::b64here(&s);
        let encrypted_parts_count = parts.iter().filter(|p| p.2).count();
        assert_eq!(encrypted_parts_count, 4);
        println!("The parsed parts=====================");
        parts.iter().for_each(|p| print!("{}", &s[p.0..p.1]));
    }

    #[test]
    fn b64here_test2() {
        let s = "Everything is in clear";
        let parts = super::b64here(&s);
        assert_eq!(parts.len(), 1);
        let t = parts.first().unwrap();
        assert_eq!(t.0, 0);
        assert_eq!(t.1, s.len());
    }

    #[test]
    fn b64here_test3() {
        let s = "JhVyXYWFhYVkprM94+hLMA== ";
        let parts = super::b64here(&s);
        parts.iter().for_each(|p| print!("[{}]", &s[p.0..p.1]));
        assert_eq!(parts.len(), 2);
    }
}

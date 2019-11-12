pub mod aes;

use crate::aes::{ aes_ctr_decrypt, aes_ctr_encrypt};

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
    // in the tuple the bool is true if we have a base64 value, opposite of Part.clear
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
        assert_eq!(parts.len(), 8);
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

pub fn memo_decrypt(encrypted_memo: &str, secret: &str) -> String {
    let parts = b64here(encrypted_memo);
    let mut result = String::with_capacity(encrypted_memo.len());
    parts.iter()
      .map(|p| Part { text: &encrypted_memo[p.0..p.1], clear: !p.2 })
      .for_each(|p| {
        if p.clear {
          result.push_str(p.text);
        } else {
          result.push('\u{300c}');
          result.push_str(&aes_ctr_decrypt(p.text, secret));
          result.push('\u{300d}');
        }        
      });
    result
}

#[cfg(test)]
mod test_memo_decryption {
  #[test]
  fn memo_decrypt_test() {
    let memo_encrypted = r#"
First secret
8xRyXaSkpKQGqlTMpMssgnNsZDnatopg

Second secret
/xRyXY6Ojo7/u45hZut8f41Uf6C2GvNCdA==
"#;
    let memo_clear = "
First secret
\u{300c}The first secret\u{300d}

Second secret
\u{300c}The second secret\u{300d}
";

  assert_eq!(memo_clear, super::memo_decrypt(memo_encrypted, "secret"));
  }
}

pub fn memo_encrypt(clear_memo: &str, secret: &str, initial_nonce: u64) -> Result<String, &'static str> {
  let opening_quote_size = "\u{300c}".len();
  let closing_quote_size = "\u{300d}".len();

  let mut nonce = initial_nonce;
  let mut result = String::with_capacity(clear_memo.len() * 2);
  let mut start = 0;
  let mut encrypt = false;

  let mut prev_char = '\n';
  let mut after_closing_quote = false;
  for (pos, c) in clear_memo.char_indices() {
      match c {
        // opening quote
        '\u{300c}' => {
            after_closing_quote = false;
            if pos > start {
              result.push_str(&clear_memo[start..pos]);
              // make sure the encrypted result does not touch the text before it
              match prev_char {
                ' '| '\n' |'\r' | '\t' => (),
                _ => result.push(' '), 
              }
            }
            start = pos + opening_quote_size;
            if encrypt {
                return Err("Previous quote was not ended");
            }
            encrypt = true;
          },
        // closing quote
        '\u{300d}' => {
            after_closing_quote = true;
            if !encrypt {
                return Err("Closing quote has no opening quote");
            }
            result.push_str(&aes_ctr_encrypt(&clear_memo[start..pos], secret, nonce));
            nonce += (2 + (pos - start) / 16) as u64;
            start = pos + closing_quote_size;
            encrypt = false;
          },
          _ => {
            // if we just added an encrypted section make sure it does not touch subsequent chars
            if after_closing_quote {
              match c {
                ' '| '\n' |'\r' | '\t' => (),
                _ => result.push(' '), 
              }
                after_closing_quote = false; 
            }
              prev_char = c
          },

      }
  }
  if start < clear_memo.len() {
    result.push_str(&clear_memo[start..]);
  }
  Ok(result)
}

#[cfg(test)]
mod test_memo_encrypt {
  #[test]
  fn memo_encrypt_test() {
    let clear_memo = "
First secret
\u{300c}The first secret\u{300d} a

Second secret
\u{300c}The second secret\u{300d}
End
";

    let encrypted_memo = "
First secret
AAAAAAAAAACmrCf4UYHplcBTiCEztS/3 a

Second secret
AwAAAAAAAADSUdXvchKudrwyi9q+mYmOUg==
End
";
    match super::memo_encrypt(&clear_memo, "secret", 0) {
      Ok(s) => assert_eq!(encrypted_memo, s),
      Err(s) => println!("Failed to process memo: {}", s),
    }
  }

  #[test]
  fn memo_encrypt_spaces() {
    let clear_memo = "
First secret\u{300c}The first secret\u{300d}
Second secret
\u{300c}The second secret\u{300d}is here
";
    let encrypted_memo = "
First secret AAAAAAAAAACmrCf4UYHplcBTiCEztS/3
Second secret
AwAAAAAAAADSUdXvchKudrwyi9q+mYmOUg== is here
";   
  
    match super::memo_encrypt(&clear_memo, "secret", 0) {
      Ok(s) => assert_eq!(encrypted_memo, s),
      Err(s) => println!("Failed to process memo: {}", s),
    }
  }

  #[test]
  fn memo_encrypt_short() {
    let clear_memo = "Secret \u{300c}secret\u{300d}";
    let encrypted_memo = "Secret AAAAAAAAAACBoSGqUpw=";

    match super::memo_encrypt(&clear_memo, "secret", 0) {
      Ok(s) => assert_eq!(encrypted_memo, s),
      Err(s) => println!("Failed to process memo: {}", s),
    }      
  }
}
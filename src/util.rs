use std::{cell::RefCell, rc::Rc};
pub use util::*;

pub type RefValue<Value> = Rc<RefCell<Value>>;
pub type OpRefValue<Value> = Option<RefValue<Value>>;

pub fn to_cyan(s: &str) -> String {
  format!("\x1b[96m{}\x1b[0m", s)
}
pub fn split_meta(meta: &str) -> (&str, &str) {
  let mut meta = meta.split("\0");
  let line = meta.next().unwrap();
  let token = meta.next();
  if token == None {
    return (line, "");
  }
  let token = token.unwrap();
  (line, token)
}

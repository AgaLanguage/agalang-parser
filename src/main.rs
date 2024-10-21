use std::process::ExitCode;

use ::util::List;
use parser::ast::{Node, NodeBlock, NodeProperty};

mod internal;
mod lexer;
mod parser;
mod util;

fn main() -> ExitCode {
    let filename = file();
    if filename.is_none() {
        return ExitCode::FAILURE;
    }
    let filename = filename.unwrap();
    let source = code(&filename);

    if source.is_none() {
        println!("Error al leer el archivo");
        return ExitCode::FAILURE;
    }

    let source = source.unwrap();
    let program = parser::Parser::new(source, &filename).produce_ast();
    if program.is_error() {
        let type_err = internal::errors::ErrorNames::SyntaxError;
        let node_err = program.get_error().unwrap();
        let err = parser::node_error(&node_err);
        let data = internal::error_to_string(&type_err, err);
        internal::print_error(data);
        return ExitCode::FAILURE;
    }
    println!("{}", json(&program));
    ExitCode::SUCCESS
}

fn json(node: &Node) -> String {
    match node {
        Node::Array(a) => format!(
            "{{\"kind\":\"Array\",\"column\":{},\"line\":{},\"file\":\"{}\",\"body\":[{}]}}",
            a.column,
            a.line,
            json_str(&a.file),
            list_property(&a.elements),
        ),
        Node::Assignment(a) => format!(
              "{{\"kind\":\"Assignment\",\"column\":{},\"line\":{},\"file\":\"{}\",\"identifier\":{},\"value\":{}}}",
              a.column,
              a.line,
              json_str(&a.file),
              json(a.identifier.as_ref()),
              json(a.value.as_ref())
            ),
        Node::Binary(b)=>format!(
          "{{\"kind\":\"Binary\",\"column\":{},\"line\":{},\"file\":\"{}\",\"left\":{},\"right\":{},\"operator\":\"{}\"}}",
          b.column,
          b.line,
          json_str(&b.file),
          json(b.left.as_ref()),
          json(b.right.as_ref()),
          json_str(&b.operator)
        ),
        Node::Block(b) => json_b(b),
        Node::Byte(_)|
        Node::Call(_)|
        Node::Class(_)|
        Node::DoWhile(_)|
        Node::Error(_)|
        Node::Export(_)|
        Node::For(_)|
        Node::Function(_)|
        Node::Identifier(_)|
        Node::If(_)|
        Node::Import(_)|
        Node::LoopEdit(_)|
        Node::Member(_)|
        Node::Name(_)|
        Node::None|
        Node::Number(_)|
        Node::Object(_)=> "null".to_string(),
        Node::Program(p)=>format!(
          "{{\"kind\":\"Program\",\"column\":{},\"line\":{},\"file\":\"{}\",\"body\":[{}]}}",
          p.column,
          p.line,
          json_str(&p.file),
          json_b(&p.body),
      ),
      Node::Return(_)|
      Node::String(_)|
      Node::Throw(_)|
      Node::Try(_)|
      Node::UnaryBack(_)|
      Node::UnaryFront(_)|
      Node::VarDecl(_)|
      Node::While(_) => "null".to_string()
    }
}
fn json_str(str: &str) -> String {
    str.replace("\n", "\\n").replace("\"", "\\\"")
}
fn json_b(b: &NodeBlock) -> String {
    format!(
        "{{\"kind\":\"Block\",\"body\":[{}],\"in_function\":{},\"in_loop\":{}}}",
        b.body.map(|n| json(n)).join(","),
        b.in_function,
        b.in_loop
    )
}
fn json_p(node_p: &NodeProperty) -> String {
    match node_p {
        NodeProperty::Dynamic(key, value) => format!(
            "{{\"kind\":\"PropertyDynamic\",\"key\":{},\"value\":{}}}",
            json(key),
            json(value)
        ),
        NodeProperty::Indexable(val) => {
            format!("{{\"kind\":\"PropertyIndexable\",\"value\":{}}}", json(val))
        }
        NodeProperty::Iterable(val) => {
            format!("{{\"kind\":\"PropertyIterable\",\"value\":{}}}", json(val))
        }
        NodeProperty::Property(key, value) => format!(
            "{{\"kind\":\"PropertyProperty\",\"key\":\"{}\",\"value\":{}}}",
            json_str(key),
            json(value)
        ),
    }
}
fn list_property(list: &List<NodeProperty>) -> String {
    list.map(|n| json_p(n)).join(",")
}

fn file() -> Option<String> {
    let mut args: Vec<_> = std::env::args().collect();
    args.push("file.agal".to_string());
    let args = args;
    if args.len() < 2 {
        let blue_usage = "\x1b[94m\x1b[1mUsage\x1b[39m:\x1b[0m";
        println!("{} {} <filename>", blue_usage, args[0]);
        return None;
    }
    Some(args[1].to_string())
}
fn code(filename: &str) -> Option<String> {
    let path = std::path::Path::new(filename);
    let contents = std::fs::read_to_string(path);
    match contents {
        Ok(contents) => Some(contents),
        Err(err) => {
            let ref type_err = internal::ErrorNames::PathError;
            let err = internal::ErrorTypes::IoError(err);
            internal::show_error(type_err, err);
            None
        }
    }
}

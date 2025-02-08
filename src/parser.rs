pub mod ast;
pub mod string;
use ast::{NodeBlock, NodeError};
use util::{List, Token};

use crate::{
  internal,
  lexer::{KeywordsType, OperatorType, PunctuationType, TokenType},
  util::{split_meta, to_cyan},
};

const ASSIGNMENT_MODIFICATOR: [&str; 8] = ["+", "-", "*", "/", "%", "&&", "||", "??"];
const COMPARISON: [&str; 5] = ["=", "~", "!", "<", ">"];
const MISSING_TOKEN: &str = "\x1b[81mToken desaparecido\x1b[0m";

struct SemiToken {
  value: String,
  location: util::Location,
}

pub fn node_error(error: &ast::NodeError) -> internal::ErrorTypes {
  let line: usize;
  let column_node: usize;
  let meta: &str;
  let message: &str;

  if error.message == MISSING_TOKEN {
    line = 0;
    column_node = 0;
    meta = &error.meta;
    message = MISSING_TOKEN;
  } else {
    line = error.location.start.line + 1;
    column_node = error.location.start.column + 1;
    meta = &error.meta;
    message = &error.message;
  }

  let (file, data_line, node_value) = split_meta(&meta);

  let column = column_node + node_value.len();

  let str_line = line.to_string();
  let str_init = " ".repeat(str_line.len());

  let cyan_line = to_cyan("|");
  let cyan_arrow = to_cyan("-->");

  let indicator = if node_value.len() > 0 {
    format!("{}^", "-".repeat(node_value.len()))
  } else {
    "^".to_string()
  };
  let lines = [
    format!("{}", message),
    format!("{}{cyan_arrow} {}:{}:{}", str_init, file, line, column),
    format!("{} {cyan_line}", str_init),
    format!("{} {cyan_line} {}", to_cyan(&str_line), data_line),
    format!(
      "{} {cyan_line} {}{}",
      str_init,
      " ".repeat(column_node - 1),
      to_cyan(&indicator)
    ),
    format!("{} {cyan_line}", str_init),
  ];
  let joined = lines.join("\n");
  internal::ErrorTypes::StringError(joined)
}

pub struct Parser {
  source: String,
  tokens: Vec<Token<TokenType>>,
  index: usize,
  file_name: String,
}
impl Parser {
  pub fn new(source: String, file_name: &str) -> Parser {
    let tokens = crate::lexer::tokenizer(source.clone(), file_name.to_string());
    Parser {
      source: source.clone(),
      tokens,
      index: 0,
      file_name: file_name.to_string(),
    }
  }
  fn not_eof(&mut self) -> bool {
    self.index < self.tokens.len()
  }
  fn prev(&self) -> util::Token<TokenType> {
    let token = self.tokens.get(self.index - 1);
    if token.is_none() {
      return util::Token::<TokenType> {
        token_type: TokenType::Error,
        value: MISSING_TOKEN.to_string(),
        location: util::Location {
          start: util::Position { line: 0, column: 0 },
          end: util::Position { line: 0, column: 0 },
          length: 0,
        },
        meta: format!("{}\0{}\0{}", &self.file_name, MISSING_TOKEN, MISSING_TOKEN),
      };
    }
    let token = token.unwrap();
    util::Token::<TokenType> {
      token_type: token.token_type,
      value: token.value.clone(),
      location: token.location,
      meta: token.meta.clone(),
    }
  }
  fn at(&self) -> util::Token<TokenType> {
    let token = self.tokens.get(self.index);
    if token.is_none() {
      let location = self.prev().location;
      let line = self.source.lines().nth(location.start.line).unwrap();
      return util::Token::<TokenType> {
        token_type: TokenType::Error,
        value: "Se esperaba un token".to_string(),
        location,
        meta: format!(
          "{}\0{}\0{}",
          &self.file_name,
          line,
          " ".repeat(location.start.column)
        ),
      };
    }
    let token = token.unwrap();
    util::Token::<TokenType> {
      token_type: token.token_type,
      value: token.value.clone(),
      location: token.location,
      meta: token.meta.clone(),
    }
  }
  fn eat(&mut self) -> util::Token<TokenType> {
    let token = self.at();
    self.index += 1;
    token
  }
  fn next(&self) -> util::Token<TokenType> {
    let token = self.tokens.get(self.index + 1);
    if token.is_none() {
      let location = self.prev().location;
      let line = self.source.lines().nth(location.start.line).unwrap();
      return util::Token::<TokenType> {
        token_type: TokenType::Error,
        value: "Se esperaba un token".to_string(),
        location,
        meta: format!(
          "{}\0{}\0{}",
          &self.file_name,
          line,
          " ".repeat(location.start.column)
        ),
      };
    }
    let token = token.unwrap();
    util::Token::<TokenType> {
      token_type: token.token_type,
      value: token.value.clone(),
      location: token.location,
      meta: token.meta.clone(),
    }
  }
  fn expect(&mut self, token_type: TokenType, err: &str) -> util::Token<TokenType> {
    let token = self.tokens.get(self.index);
    self.index += 1;
    if token.is_none() {
      let location = self.prev().location;
      let line = self.source.lines().nth(location.start.line).unwrap();
      return util::Token::<TokenType> {
        token_type: TokenType::Error,
        value: err.to_string(),
        location,
        meta: format!(
          "{}\0{}\0{}",
          &self.file_name,
          line,
          " ".repeat(location.start.column)
        ),
      };
    }
    let token = token.unwrap();
    if token.token_type != token_type {
      let line = self.source.lines().nth(token.location.start.line).unwrap();
      return util::Token::<TokenType> {
        token_type: TokenType::Error,
        value: err.to_string(),
        location: token.location,
        meta: format!(
          "{}\0{}\0{}",
          &self.file_name,
          line,
          " ".repeat(token.location.start.column)
        ),
      };
    }
    util::Token::<TokenType> {
      token_type: token.token_type,
      value: token.value.clone(),
      location: token.location,
      meta: token.meta.clone(),
    }
  }
  pub fn produce_ast(&mut self) -> Result<ast::Node, NodeError> {
    let body = self.parse_block(true, false, false, true, TokenType::EOF)?;
    let location = body.location;
    ast::Node::Program(ast::NodeProgram {
      body,
      location,
      file: self.file_name.clone(),
    })
    .into()
  }
  fn parse_stmt(
    &mut self,
    is_global_scope: bool,
    is_function: bool,
    is_loop: bool,
    is_async: bool,
  ) -> Option<Result<ast::Node, NodeError>> {
    let token = self.at();
    match token.token_type {
      TokenType::EOF => {
        self.eat();
        None
      }
      TokenType::Error => {
        return Some(Err(ast::NodeError {
          message: token.value,
          location: token.location,
          meta: token.meta,
        }));
      }
      TokenType::Keyword(key) => match key {
        KeywordsType::Define | KeywordsType::Constant => Some(self.parse_var_decl()),
        KeywordsType::While
        | KeywordsType::Do
        | KeywordsType::If
        | KeywordsType::Function
        | KeywordsType::Try
        | KeywordsType::Class
        | KeywordsType::Para
        | KeywordsType::Async => Some(self.parse_keyword_value(is_function, is_loop, true)),
        KeywordsType::Return | KeywordsType::Continue | KeywordsType::Romper => {
          Some(self.parse_simple_decl(is_function, is_loop))
        }
        KeywordsType::Export => Some(self.parse_export_decl(is_global_scope)),
        KeywordsType::Import => Some(self.parse_import_decl(is_global_scope)),
        KeywordsType::Throw => Some(self.parse_throw_decl()),
        KeywordsType::Await => {
          self.eat(); // await
          if !is_async {
            return Some(Err(ast::NodeError {
              message: format!(
                "La palabra clave '{}' solo se puede utilizar en un contexto asíncrono",
                KeywordsType::Await.as_str()
              ),
              location: token.location,
              meta: token.meta,
            }));
          }
          Some(
            ast::Node::Await(ast::NodeExpressionMedicator {
              expression: match self.parse_stmt_expr() {
                Err(e) => return Some(Err(e)),
                Ok(expr) => expr.to_box(),
              },
              location: token.location,
              file: token.meta,
            })
            .into(),
          )
        }
        _ => {
          self.eat();
          None
        }
      },
      _ => Some(self.parse_stmt_expr()),
    }
  }
  fn parse_throw_decl(&mut self) -> Result<ast::Node, NodeError> {
    let token = self.eat(); // lanzar
    let expr = self.parse_expr()?;
    let semicolon = self.expect(TokenType::Punctuation(PunctuationType::SemiColon), "");
    if semicolon.token_type == TokenType::Error {
      return Err(ast::NodeError {
        message: format!(
          "Se esperaba un punto y coma ({})",
          KeywordsType::Throw.to_string()
        ),
        location: semicolon.location,
        meta: semicolon.meta,
      });
    }
    ast::Node::Throw(ast::NodeValue {
      value: Box::new(expr),
      location: token.location,
      file: token.meta,
    })
    .into()
  }
  fn parse_import_decl(&mut self, is_global_scope: bool) -> Result<ast::Node, NodeError> {
    let token = self.eat(); // importar
    let path = self.expect(
      TokenType::StringLiteral,
      "Se esperaba una ruta de archivo, debe usar una cadena literal con '",
    );
    if path.token_type == TokenType::Error {
      return Err(ast::NodeError {
        message: path.value.clone(),
        location: path.location,
        meta: path.meta,
      });
    }
    let mut is_lazy = false;
    let mut name = None;
    if self.at().token_type == TokenType::Keyword(KeywordsType::As) {
      self.eat();
      if self.at().token_type == TokenType::Keyword(KeywordsType::Lazy) {
        self.eat();
        is_lazy = true;
      }
      let alias = self.expect(TokenType::Identifier, "Se esperaba un identificador");
      if alias.token_type == TokenType::Error {
        return Err(ast::NodeError {
          message: alias.value.clone(),
          location: alias.location,
          meta: alias.meta,
        });
      }
      name = Some(alias.value.clone());
    }
    let semicolon = self.expect(TokenType::Punctuation(PunctuationType::SemiColon), "");
    if semicolon.token_type == TokenType::Error {
      return Err(ast::NodeError {
        message: format!("Se esperaba un punto y coma ({})", path.value),
        location: semicolon.location,
        meta: semicolon.meta,
      });
    }
    if !is_global_scope {
      let line = self.source.lines().nth(token.location.start.line).unwrap();
      return Err(ast::NodeError {
        message: "No se puede importar fuera del ámbito global".to_string(),
        location: token.location,
        meta: format!("{}\0{}\0{}", &self.file_name, line, token.value),
      });
    }
    ast::Node::Import(ast::NodeImport {
      path: path.value.clone(),
      name,
      is_lazy,
      location: token.location,
      file: token.meta.clone(),
    })
    .into()
  }
  fn parse_export_decl(&mut self, is_global_scope: bool) -> Result<ast::Node, NodeError> {
    let token = self.eat(); // exportar
    let value = self.parse_export_value()?;
    if !is_global_scope {
      let line = self.source.lines().nth(token.location.start.line).unwrap();
      let error = ast::NodeError {
        message: "No se puede exportar fuera del ámbito global".to_string(),
        location: token.location,
        meta: format!("{}\0{}\0{}", &self.file_name, line, token.value),
      };
      let type_err = internal::ErrorNames::SyntaxError;
      let err = node_error(&error);
      let data = internal::error_to_string(&type_err, err);
      internal::print_warn(data);
      return Ok(value);
    }
    ast::Node::Export(ast::NodeValue {
      value: Box::new(value),
      location: token.location,
      file: token.meta.clone(),
    })
    .into()
  }
  fn parse_export_value(&mut self) -> Result<ast::Node, NodeError> {
    let token = self.at();
    match token.token_type {
      TokenType::Keyword(KeywordsType::Define | KeywordsType::Constant) => self.parse_var_decl(),
      TokenType::Keyword(KeywordsType::Function | KeywordsType::Class) => {
        self.parse_keyword_value(false, false, false)
      }
      TokenType::Keyword(KeywordsType::Name) => self.parse_name_decl(),
      _ => {
        self.eat();
        let line = self.source.lines().nth(token.location.start.line).unwrap();
        let (message, value) = if token.token_type == TokenType::Error {
          (token.value, "".to_string())
        } else {
          ("Se esperaba un valor exportable".to_string(), token.value)
        };
        Err(ast::NodeError {
          message,
          location: token.location,
          meta: format!("{}\0{}\0{}", &self.file_name, line, value),
        })
      }
    }
  }
  fn parse_name_decl(&mut self) -> Result<ast::Node, NodeError> {
    let token = self.eat(); // nombre
    let name = self.expect(TokenType::Identifier, "Se esperaba un identificador");
    if name.token_type == TokenType::Error {
      return Err(ast::NodeError {
        message: name.value.clone(),
        location: name.location,
        meta: name.meta,
      });
    }
    let semicolon = self.expect(TokenType::Punctuation(PunctuationType::SemiColon), "");
    if semicolon.token_type == TokenType::Error {
      return Err(ast::NodeError {
        message: format!(
          "Se esperaba un punto y coma ({})",
          KeywordsType::Name.to_string()
        ),
        location: semicolon.location,
        meta: semicolon.meta,
      });
    }
    ast::Node::Name(ast::NodeIdentifier {
      name: name.value.clone(),
      location: token.location,
      file: token.meta,
    })
    .into()
  }
  fn parse_class_prop(
    &mut self,
    is_static: bool,
    is_public: bool,
  ) -> Result<ast::NodeClassProperty, ast::NodeError> {
    let is_async = if self.at().token_type == TokenType::Keyword(KeywordsType::Async) {
      self.eat();
      true
    } else {
      false
    };
    let name = self.expect(TokenType::Identifier, "Se esperaba un identificador");
    if name.token_type == TokenType::Error {
      return Err(ast::NodeError {
        message: name.value.clone(),
        location: name.location,
        meta: name.meta,
      });
    }
    let token = self.at();
    if token.token_type == TokenType::Error {
      self.eat();
      return Err(ast::NodeError {
        message: token.value.clone(),
        location: token.location,
        meta: token.meta,
      });
    }
    let is_static_bit: u8 = if is_static { 1 } else { 0 };
    let is_public_bit: u8 = if is_public { 1 << 1 } else { 0 };
    let meta: u8 = is_static_bit | is_public_bit;
    if token.token_type == TokenType::Punctuation(PunctuationType::SemiColon) {
      return Ok(ast::NodeClassProperty {
        name: name.value.clone(),
        value: None,
        meta,
      });
    }
    let value: ast::Node =
      if token.token_type == TokenType::Punctuation(PunctuationType::CircularBracketOpen) {
        let params = self.parse_arguments_expr()?;
        let body = self.parse_block_expr(true, false, is_async)?;
        ast::Node::Function(ast::NodeFunction {
          is_async,
          name: name.value.clone(),
          params,
          body,
          location: token.location,
          file: token.meta.clone(),
        })
      } else if token.token_type == TokenType::Operator(OperatorType::Equals) {
        self.eat();
        self.parse_expr()?
      } else {
        let line = self.source.lines().nth(token.location.start.line).unwrap();
        return Err(ast::NodeError {
          message: "Se esperaba un valor".to_string(),
          location: token.location,
          meta: format!("{}\0{}\0{}", &self.file_name, line, token.value),
        });
      };
    let semicolon = self.expect(TokenType::Punctuation(PunctuationType::SemiColon), "");
    if semicolon.token_type == TokenType::Error {
      return Err(ast::NodeError {
        message: format!("Se esperaba un punto y coma ({})", name.value),
        location: semicolon.location,
        meta: semicolon.meta,
      });
    }
    Ok(ast::NodeClassProperty {
      name: name.value.clone(),
      value: Some(value.to_box()),
      meta,
    })
  }
  fn parse_class_decl(&mut self) -> Result<ast::Node, NodeError> {
    let token = self.eat(); // clase
    let name = self.expect(TokenType::Identifier, "Se esperaba un identificador");
    if name.token_type == TokenType::Error {
      return Err(ast::NodeError {
        message: name.value.clone(),
        location: name.location,
        meta: name.meta,
      });
    }

    let extend_of = if self.at().token_type == TokenType::Keyword(KeywordsType::Extender) {
      self.eat();
      let class_node = match self.parse_literal_expr() {
        Ok(node) => node,
        Err(token) => Err(ast::NodeError {
          message: token.value,
          location: token.location,
          meta: token.meta,
        }),
      }?;
      if let ast::Node::Identifier(id) = class_node {
        Some(id)
      } else {
        return Err(ast::NodeError {
          message: "Se esperaba un identificador".to_string(),
          location: class_node.get_location(),
          meta: class_node.get_file(),
        });
      }
    } else {
      None
    };

    let open_brace = self.expect(
      TokenType::Punctuation(PunctuationType::RegularBracketOpen),
      "",
    );
    if open_brace.token_type == TokenType::Error {
      return Err(ast::NodeError {
        message: "Se esperaba un corchete de apertura".to_string(),
        location: open_brace.location,
        meta: open_brace.meta,
      });
    }
    let mut body: List<ast::NodeClassProperty> = List::new();
    while self.not_eof()
      && !(self.at().token_type == TokenType::Punctuation(PunctuationType::RegularBracketClose))
    {
      let modifier = self.get_modifier();

      if modifier.is_err() {
        return Err(modifier.err().unwrap());
      }

      let (is_static, is_public) = modifier.ok().unwrap();

      let prop = self.parse_class_prop(is_static, is_public);
      if prop.is_err() {
        return Err(prop.err().unwrap());
      }
      let prop = prop.ok().unwrap();
      body.push(prop);
    }
    let close_brace = self.expect(
      TokenType::Punctuation(PunctuationType::RegularBracketClose),
      "",
    );
    if close_brace.token_type == TokenType::Error {
      return Err(ast::NodeError {
        message: "Se esperaba un corchete de cierre".to_string(),
        location: close_brace.location,
        meta: close_brace.meta,
      });
    }
    ast::Node::Class(ast::NodeClass {
      name: name.value.clone(),
      extend_of,
      body,
      location: token.location,
      file: token.meta,
    })
    .into()
  }
  fn get_modifier(&mut self) -> Result<(bool, bool), ast::NodeError> {
    let mut is_static = false;
    let mut is_public = false;
    while self.not_eof() {
      let token = self.at();
      if token.token_type == TokenType::Error {
        self.eat();
        return Err(ast::NodeError {
          message: token.value.clone(),
          location: token.location,
          meta: token.meta,
        });
      }
      if token.token_type == TokenType::Keyword(KeywordsType::Static) {
        if is_static {
          let line = self.source.lines().nth(token.location.start.line).unwrap();
          return Err(ast::NodeError {
            message: "Modificador duplicado".to_string(),
            location: token.location,
            meta: format!("{}\0{}\0{}", &self.file_name, line, token.value),
          });
        }
        is_static = true;
        self.eat();
        continue;
      }
      if token.token_type == TokenType::Keyword(KeywordsType::Public) {
        if is_public {
          let line = self.source.lines().nth(token.location.start.line).unwrap();
          return Err(ast::NodeError {
            message: "Modificador duplicado".to_string(),
            location: token.location,
            meta: format!("{}\0{}\0{}", &self.file_name, line, token.value),
          });
        }
        is_public = true;
        self.eat();
        continue;
      }
      break;
    }
    Ok((is_static, is_public))
  }
  fn parse_simple_decl(
    &mut self,
    is_function: bool,
    is_loop: bool,
  ) -> Result<ast::Node, NodeError> {
    let token = self.eat(); // TokenType::Keyword
                            // cont rom ret
    match token.token_type {
      TokenType::Keyword(KeywordsType::Return) => {
        if !is_function {
          let line = self.source.lines().nth(token.location.start.line).unwrap();
          return Err(ast::NodeError {
            message: "No se puede retornar fuera de una función".to_string(),
            location: token.location,
            meta: format!("{}\0{}\0{}", &self.file_name, line, token.value),
          });
        }
        let expr = self.parse_expr()?;
        let semicolon = self.expect(TokenType::Punctuation(PunctuationType::SemiColon), "");
        if semicolon.token_type == TokenType::Error {
          return Err(ast::NodeError {
            message: format!(
              "Se esperaba un punto y coma ({})",
              KeywordsType::Return.to_string()
            ),
            location: semicolon.location,
            meta: semicolon.meta,
          });
        }
        ast::Node::Return(ast::NodeReturn {
          value: Some(expr.to_box()),
          location: token.location,
          file: token.meta,
        })
        .into()
      }
      TokenType::Keyword(KeywordsType::Romper | KeywordsType::Continue) => {
        if !is_loop {
          let line = self.source.lines().nth(token.location.start.line).unwrap();
          return Err(ast::NodeError {
            message: "No se puede usar esta palabra clave fuera de un ciclo".to_string(),
            location: token.location,
            meta: format!("{}\0{}\0{}", &self.file_name, line, token.value),
          });
        }
        let semicolon = self.expect(TokenType::Punctuation(PunctuationType::SemiColon), "");
        if semicolon.token_type == TokenType::Error {
          return Err(ast::NodeError {
            message: format!("Se esperaba un punto y coma (Modificador de Bucle)"),
            location: semicolon.location,
            meta: semicolon.meta,
          });
        }
        let action = if token.value == "cont" {
          ast::NodeLoopEditType::Continue
        } else {
          ast::NodeLoopEditType::Break
        };
        ast::Node::LoopEdit(ast::NodeLoopEdit {
          action,
          location: token.location,
          file: token.meta,
        })
        .into()
      }
      TokenType::Error => Err(ast::NodeError {
        message: token.value.clone(),
        location: token.location,
        meta: token.meta,
      }),
      _ => {
        let line = self.source.lines().nth(token.location.start.line).unwrap();
        return Err(ast::NodeError {
          message: "Token inesperado (simple)".to_string(),
          location: token.location,
          meta: format!("{}\0{}\0{}", &self.file_name, line, token.value),
        });
      }
    }
  }
  fn parse_keyword_value(
    &mut self,
    is_function: bool,
    is_loop: bool,
    is_async: bool,
  ) -> Result<ast::Node, NodeError> {
    let token = self.at();
    match token.token_type {
      TokenType::Keyword(KeywordsType::Para) => self.parse_for_decl(is_function, is_async),
      TokenType::Keyword(KeywordsType::While) => self.parse_while_decl(is_function, is_async),
      TokenType::Keyword(KeywordsType::Do) => self.parse_do_while_decl(is_function, is_async),
      TokenType::Keyword(KeywordsType::If) => self.parse_if_decl(is_function, is_loop, is_async),
      TokenType::Keyword(KeywordsType::Function) => self.parse_function_decl(false),
      TokenType::Keyword(KeywordsType::Async) => {
        self.eat();
        self.parse_function_decl(true)
      }
      TokenType::Keyword(KeywordsType::Try) => self.parse_try_decl(is_function, is_loop, is_async),
      TokenType::Keyword(KeywordsType::Class) => self.parse_class_decl(),
      TokenType::Error => {
        self.eat();
        Err(ast::NodeError {
          message: token.value.clone(),
          location: token.location,
          meta: token.meta,
        })
      }
      _ => {
        let line = self.source.lines().nth(token.location.start.line).unwrap();
        Err(ast::NodeError {
          message: "Token inesperado (keyword)".to_string(),
          location: token.location,
          meta: format!("{}\0{}\0{}", &self.file_name, line, token.value),
        })
      }
    }
  }
  fn parse_for_decl(&mut self, is_function: bool, is_async: bool) -> Result<ast::Node, NodeError> {
    let token = self.eat(); // para
    let open_paren = self.expect(
      TokenType::Punctuation(PunctuationType::CircularBracketOpen),
      "",
    );
    if open_paren.token_type == TokenType::Error {
      return Err(ast::NodeError {
        message: "Se esperaba un paréntesis de apertura".to_string(),
        location: open_paren.location,
        meta: open_paren.meta,
      });
    }
    let init = self.parse_var_decl()?.to_box();
    let condition = self.parse_expr()?;
    let semicolon = self.expect(TokenType::Punctuation(PunctuationType::SemiColon), "");
    if semicolon.token_type == TokenType::Error {
      return Err(ast::NodeError {
        message: "Se esperaba un punto y coma (Para)".to_string(),
        location: semicolon.location,
        meta: semicolon.meta,
      });
    }
    let update = self.parse_expr()?;
    let close_paren = self.expect(
      TokenType::Punctuation(PunctuationType::CircularBracketClose),
      "",
    );
    if close_paren.token_type == TokenType::Error {
      return Err(ast::NodeError {
        message: "Se esperaba un paréntesis de cierre".to_string(),
        location: close_paren.location,
        meta: close_paren.meta,
      });
    }
    let block = self.parse_block_expr(is_function, true, is_async);
    if block.is_err() {
      return Err(block.err().unwrap());
    }
    let body = block.ok().unwrap();
    ast::Node::For(ast::NodeFor {
      init,
      condition: Box::new(condition),
      update: Box::new(update),
      body,
      location: token.location,
      file: token.meta,
    })
    .into()
  }
  fn parse_try_decl(
    &mut self,
    is_function: bool,
    is_loop: bool,
    is_async: bool,
  ) -> Result<ast::Node, NodeError> {
    let token = self.eat(); // intentar
    let block = self.parse_block_expr(is_function, is_loop, is_async);
    if block.is_err() {
      return Err(block.err().unwrap());
    }
    let body = block.ok().unwrap();
    let catch = if self.at().token_type == TokenType::Keyword(KeywordsType::Catch) {
      self.eat();
      let open_paren = self.expect(
        TokenType::Punctuation(PunctuationType::CircularBracketOpen),
        "",
      );
      if open_paren.token_type == TokenType::Error {
        return Err(ast::NodeError {
          message: "Se esperaba un paréntesis de apertura".to_string(),
          location: open_paren.location,
          meta: open_paren.meta,
        });
      }
      let identifier = self.expect(TokenType::Identifier, "");
      if identifier.token_type == TokenType::Error {
        return Err(ast::NodeError {
          message: identifier.value.clone(),
          location: identifier.location,
          meta: identifier.meta,
        });
      }
      let close_paren = self.expect(
        TokenType::Punctuation(PunctuationType::CircularBracketClose),
        "",
      );
      if close_paren.token_type == TokenType::Error {
        return Err(ast::NodeError {
          message: "Se esperaba un paréntesis de cierre".to_string(),
          location: close_paren.location,
          meta: close_paren.meta,
        });
      }
      let block = self.parse_block_expr(is_function, is_loop, is_async);
      if block.is_err() {
        return Err(block.err().unwrap());
      }
      Some((identifier.value.clone(), block.ok().unwrap()))
    } else {
      None
    };
    let finally = if self.at().token_type == TokenType::Keyword(KeywordsType::Finally) {
      self.eat();
      let block = self.parse_block_expr(is_function, is_loop, is_async);
      if block.is_err() {
        return Err(block.err().unwrap());
      }
      Some(block.ok().unwrap())
    } else {
      None
    };
    ast::Node::Try(ast::NodeTry {
      body,
      catch,
      finally,
      location: token.location,
      file: token.meta,
    })
    .into()
  }
  fn parse_function_decl(&mut self, is_async: bool) -> Result<ast::Node, NodeError> {
    let token = self.eat(); // fn
    let name = self.expect(TokenType::Identifier, "Se esperaba un identificador");
    if name.token_type == TokenType::Error {
      return Err(ast::NodeError {
        message: name.value.clone(),
        location: name.location,
        meta: name.meta,
      });
    }
    let params = self.parse_arguments_expr();
    if params.is_err() {
      return Err(params.err().unwrap());
    }
    let params = params.ok().unwrap();
    let block = self.parse_block_expr(true, false, is_async);
    if block.is_err() {
      return Err(block.err().unwrap());
    }
    let body = block.ok().unwrap();
    ast::Node::Function(ast::NodeFunction {
      is_async,
      name: name.value.clone(),
      params,
      body,
      location: token.location,
      file: token.meta,
    })
    .into()
  }
  fn parse_arguments_expr(&mut self) -> Result<List<ast::NodeIdentifier>, ast::NodeError> {
    let open_paren = self.expect(
      TokenType::Punctuation(PunctuationType::CircularBracketOpen),
      "",
    );
    if open_paren.token_type == TokenType::Error {
      return Err(ast::NodeError {
        message: "Se esperaba un paréntesis de apertura".to_string(),
        location: open_paren.location,
        meta: open_paren.meta,
      });
    }
    let mut params = List::new();
    while self.not_eof()
      && !(self.at().token_type == TokenType::Punctuation(PunctuationType::CircularBracketClose))
    {
      let param = self.expect(TokenType::Identifier, "Se esperaba un identificador");
      if param.token_type == TokenType::Error {
        return Err(ast::NodeError {
          message: param.value.clone(),
          location: param.location,
          meta: param.meta,
        });
      }
      params.push(ast::NodeIdentifier {
        name: param.value.clone(),
        location: param.location,
        file: param.meta,
      });
      let comma = self.at();
      if comma.token_type == TokenType::Punctuation(PunctuationType::Comma) {
        self.eat();
        continue;
      }
      if comma.token_type == TokenType::Punctuation(PunctuationType::CircularBracketClose) {
        break;
      }
      let line = self.source.lines().nth(comma.location.start.line).unwrap();
      return Err(ast::NodeError {
        message: "Se esperaba una coma".to_string(),
        location: comma.location,
        meta: format!("{}\0{}\0{}", self.file_name, line, comma.value),
      });
    }
    let close_paren = self.expect(
      TokenType::Punctuation(PunctuationType::CircularBracketClose),
      "Se esperaba un paréntesis de cierre",
    );
    if close_paren.token_type == TokenType::Error {
      return Err(ast::NodeError {
        message: "Se esperaba un paréntesis de cierre".to_string(),
        location: close_paren.location,
        meta: close_paren.meta,
      });
    }
    Ok(params)
  }
  fn parse_if_decl(
    &mut self,
    is_function: bool,
    is_loop: bool,
    is_async: bool,
  ) -> Result<ast::Node, NodeError> {
    let token = self.eat(); // si
    let condition = self.parse_expr()?;
    let body = self.parse_block_expr(is_function, is_loop, is_async)?;
    let else_token = self.at(); // ent
    if else_token.token_type == TokenType::Keyword(KeywordsType::Else) {
      self.eat();
      let else_block = self.parse_block_expr(is_function, is_loop, is_async);
      if else_block.is_err() {
        return Err(else_block.err().unwrap());
      }
      let else_body = else_block.ok().unwrap();
      let else_body = if else_body.len() == 0 {
        None
      } else {
        Some(else_body)
      };
      return ast::Node::If(ast::NodeIf {
        condition: condition.to_box(),
        body,
        else_body,
        location: token.location,
        file: token.meta,
      })
      .into();
    }
    ast::Node::If(ast::NodeIf {
      condition: condition.to_box(),
      body,
      else_body: None,
      location: token.location,
      file: token.meta,
    })
    .into()
  }
  fn parse_do_while_decl(
    &mut self,
    is_function: bool,
    is_async: bool,
  ) -> Result<ast::Node, NodeError> {
    let token = self.eat(); // hacer
    let body = self.parse_block_expr(is_function, true, is_async)?;
    let while_token = self.expect(
      TokenType::Keyword(KeywordsType::While),
      "Se esperaba la palabra clave 'mien'",
    );
    if while_token.token_type == TokenType::Error {
      return Err(ast::NodeError {
        message: "Se esperaba la palabra clave 'mien'".to_string(),
        location: while_token.location,
        meta: while_token.meta,
      });
    }
    let condition = self.parse_expr()?.to_box();
    let semicolon = self.expect(TokenType::Punctuation(PunctuationType::SemiColon), "");
    if semicolon.token_type == TokenType::Error {
      return Err(ast::NodeError {
        message: format!(
          "Se esperaba un punto y coma ({})",
          KeywordsType::Do.to_string()
        ),
        location: semicolon.location,
        meta: semicolon.meta,
      });
    }
    ast::Node::DoWhile(ast::NodeWhile {
      condition,
      body,
      location: token.location,
      file: token.meta,
    })
    .into()
  }
  fn parse_while_decl(
    &mut self,
    is_function: bool,
    is_async: bool,
  ) -> Result<ast::Node, NodeError> {
    let token = self.eat(); // mien
    let condition = self.parse_expr()?;
    let block = self.parse_block_expr(is_function, true, is_async);
    if block.is_err() {
      return Err(block.err().unwrap());
    }
    let body = block.ok().unwrap();
    ast::Node::While(ast::NodeWhile {
      condition: condition.to_box(),
      body,
      location: token.location,
      file: token.meta,
    })
    .into()
  }
  fn parse_block_expr(
    &mut self,
    in_function: bool,
    in_loop: bool,
    is_async: bool,
  ) -> Result<NodeBlock, ast::NodeError> {
    let open_brace = self.at();
    if open_brace.token_type == TokenType::Error {
      return Err(ast::NodeError {
        message: "Se esperaba un bloque".to_string(),
        location: open_brace.location,
        meta: open_brace.meta,
      });
    }
    if self.at().token_type != TokenType::Punctuation(PunctuationType::RegularBracketOpen) {
      let expr = self.parse_stmt(false, in_function, in_loop, is_async);
      if expr.is_none() {
        let line = self
          .source
          .lines()
          .nth(open_brace.location.start.line)
          .unwrap();
        return Err(ast::NodeError {
          message: "Se esperaba un bloque".to_string(),
          location: open_brace.location,
          meta: format!("{}\0{}", self.file_name, line),
        });
      }
      let expr = expr.unwrap()?;
      let mut body = List::new();
      let location = expr.get_location();
      body.push(expr);
      return Ok(ast::NodeBlock {
        body,
        in_function,
        in_loop,
        location,
      });
    }
    self.eat();
    let body = self.parse_block(
      false,
      in_function,
      in_loop,
      is_async,
      TokenType::Punctuation(PunctuationType::RegularBracketClose),
    );
    if body.is_err() {
      return Err(body.err().unwrap());
    }
    let body = body.ok().unwrap();
    let close_brace = self.expect(
      TokenType::Punctuation(PunctuationType::RegularBracketClose),
      "",
    );
    if close_brace.token_type == TokenType::Error {
      return Err(ast::NodeError {
        message: "Se esperaba un corchete de cierre".to_string(),
        location: close_brace.location,
        meta: close_brace.meta,
      });
    }
    Ok(body)
  }
  fn parse_block(
    &mut self,
    is_global_scope: bool,
    is_function: bool,
    is_loop: bool,
    is_async: bool,
    stop_with: TokenType,
  ) -> Result<NodeBlock, ast::NodeError> {
    let mut body = List::new();
    let mut functions = Vec::new();
    let mut code = Vec::new();
    while self.not_eof() && !(self.at().token_type == stop_with) {
      let stmt = self.parse_stmt(is_global_scope, is_function, is_loop, is_async);
      if let Some(stmt) = stmt {
        let stmt = stmt?;
        match stmt {
          ast::Node::Function(_) => functions.push(stmt),
          ast::Node::Export(ref export) => match *export.value {
            ast::Node::Function(_) => functions.push(stmt.clone()),
            _ => code.push(stmt),
          },
          _ => code.push(stmt),
        }
      }
    }
    body.append_vec(&mut functions);
    body.append_vec(&mut code);
    Ok(ast::NodeBlock {
      body,
      in_function: false,
      in_loop: false,
      location: util::Location {
        start: util::Position { line: 0, column: 0 },
        end: util::Position { line: 0, column: 0 },
        length: 0,
      },
    })
  }
  fn parse_var_decl(&mut self) -> Result<ast::Node, NodeError> {
    let token = self.eat();
    let is_const = token.value == "const";
    let mut semi_token = SemiToken {
      value: token.value,
      location: token.location,
    };

    let identifier = self.expect(TokenType::Identifier, "Se esperaba un identificador");
    if semi_token.location.start.line == identifier.location.start.line {
      semi_token.value += " "
        .repeat(identifier.location.start.column - semi_token.location.start.column)
        .as_str();
    } else {
      semi_token.value = "".to_string();
    };
    semi_token.location.start.line = identifier.location.start.line;
    semi_token.location.start.column = identifier.location.start.column;
    if identifier.token_type == TokenType::Error {
      let line = self
        .source
        .lines()
        .nth(semi_token.location.start.line)
        .unwrap();
      let meta = format!("{}\0{}\0{}", self.file_name, line, semi_token.value);
      return Err(ast::NodeError {
        message: identifier.value,
        location: semi_token.location,
        meta,
      });
    }
    semi_token.value += identifier.value.as_str();

    let equals_semicolon = self.eat();
    if semi_token.location.start.line == equals_semicolon.location.start.line {
      semi_token.value += " "
        .repeat(equals_semicolon.location.start.column - semi_token.location.start.column)
        .as_str();
    } else {
      semi_token.value = "".to_string();
    };
    semi_token.location.start.line = equals_semicolon.location.start.line;
    semi_token.location.start.column = equals_semicolon.location.start.column;
    if equals_semicolon.token_type == TokenType::Punctuation(PunctuationType::SemiColon) {
      return ast::Node::VarDecl(ast::NodeVarDecl {
        name: identifier.value.clone(),
        value: None,
        is_const,
        location: identifier.location,
        file: identifier.meta,
      })
      .into();
    }
    if equals_semicolon.token_type != TokenType::Operator(OperatorType::Equals) {
      let line = self
        .source
        .lines()
        .nth(semi_token.location.start.line)
        .unwrap();
      let meta = format!("{}\0{}\0{}", self.file_name, line, semi_token.value);
      return Err(ast::NodeError {
        message: format!("Se esperaba un punto y coma (variable e)"),
        location: semi_token.location,
        meta,
      });
    }
    semi_token.value += equals_semicolon.value.as_str();

    let value = self.parse_expr()?;
    if semi_token.location.start.line == value.get_location().start.line {
      semi_token.value += " "
        .repeat(value.get_location().start.column - semi_token.location.start.column)
        .as_str();
    } else {
      semi_token.value = "".to_string();
    };
    semi_token.location.start.column = value.get_location().start.column;
    let semicolon = self.expect(TokenType::Punctuation(PunctuationType::SemiColon), "");
    if semi_token.location.start.line == semicolon.location.start.line {
      semi_token.value += " "
        .repeat(semicolon.location.start.column - semi_token.location.start.column)
        .as_str();
    } else {
      semi_token.value = "".to_string();
    };
    semi_token.location.start.column += 1;
    if semicolon.token_type == TokenType::Error {
      return Err(ast::NodeError {
        message: format!("Se esperaba un punto y coma (variable v)"),
        location: semi_token.location,
        meta: semi_token.value,
      });
    }
    ast::Node::VarDecl(ast::NodeVarDecl {
      name: identifier.value.clone(),
      value: Some(value.to_box()),
      is_const,
      location: token.location,
      file: token.meta,
    })
    .into()
  }
  fn parse_stmt_expr(&mut self) -> Result<ast::Node, NodeError> {
    let node = self.parse_expr();
    let semicolon = self.expect(
      TokenType::Punctuation(PunctuationType::SemiColon),
      "Se esperaba un punto y coma",
    );
    if semicolon.token_type == TokenType::Error {
      return Err(ast::NodeError {
        message: semicolon.value,
        location: semicolon.location,
        meta: semicolon.meta,
      });
    }
    node
  }
  fn parse_expr(&mut self) -> Result<ast::Node, NodeError> {
    let left = self.parse_math_lineal_expr()?;
    let operator_t = self.at();
    let operator: String = if let TokenType::Operator(_) = operator_t.token_type {
      self.eat().value
    } else {
      "".to_string()
    };
    self.parse_complex_expr(
      left,
      SemiToken {
        value: operator,
        location: operator_t.location,
      },
    )
  }
  fn parse_math_lineal_expr(&mut self) -> Result<ast::Node, NodeError> {
    let mut left = self.parse_math_multiplicative_expr()?;
    loop {
      let token = self.at();
      if let TokenType::Operator(OperatorType::Plus | OperatorType::Minus) = token.token_type {
        if self.next().token_type == TokenType::Operator(OperatorType::Equals) {
          return Ok(left);
        }
      } else {
        return Ok(left);
      }
      let operator = self.eat().value;
      let right = self.parse_math_multiplicative_expr()?;
      left = ast::Node::Binary(ast::NodeBinary {
        operator,
        left: left.clone().to_box(),
        right: right.to_box(),
        location: left.get_location(),
        file: left.get_file(),
      });
    }
  }
  fn parse_math_multiplicative_expr(&mut self) -> Result<ast::Node, NodeError> {
    let mut left = self.parse_math_exponential_expr()?;
    loop {
      let token = self.at();

      if let TokenType::Operator(
        OperatorType::Star | OperatorType::Division | OperatorType::Module,
      ) = token.token_type
      {
        if self.next().token_type == TokenType::Operator(OperatorType::Equals) {
          return Ok(left);
        }
      } else {
        return Ok(left);
      }
      let operator = self.eat().value;
      let right = self.parse_math_exponential_expr()?;
      left = ast::Node::Binary(ast::NodeBinary {
        operator,
        left: left.clone().to_box(),
        right: right.to_box(),
        location: left.get_location(),
        file: left.get_file(),
      })
    }
  }
  fn parse_math_exponential_expr(&mut self) -> Result<ast::Node, NodeError> {
    let left = self.parse_literal_expr();

    let left = match left {
      Ok(node) => node?,
      Err(token) => {
        let line = self.source.lines().nth(token.location.start.line).unwrap();
        return Err(ast::NodeError {
          message: "Token inesperado (exponencial iz)".to_string(),
          location: token.location,
          meta: format!("{}\0{}\0{}", token.meta, line, token.value),
        });
      }
    };
    let token = self.at();
    if token.token_type != TokenType::Operator(OperatorType::Power)
      || self.next().token_type == TokenType::Operator(OperatorType::Equals)
    {
      return left.into();
    }
    let operator = self.eat().value;
    let right = self.parse_literal_expr();
    if right.is_err() {
      let token = right.err().unwrap();
      let line = self.source.lines().nth(token.location.start.line).unwrap();
      return Err(ast::NodeError {
        message: "Token inesperado (exponencial de)".to_string(),
        location: token.location,
        meta: format!("{}\0{}\0{}", token.meta, line, token.value),
      });
    }
    let right = right.ok().unwrap()?;
    ast::Node::Binary(ast::NodeBinary {
      operator,
      left: left.clone().to_box(),
      right: right.to_box(),
      location: left.get_location(),
      file: left.get_file(),
    })
    .into()
  }
  fn parse_assignment_expr(
    &mut self,
    left: ast::Node,
    operator_st: SemiToken,
  ) -> Result<ast::Node, NodeError> {
    let token = self.at();
    let pre_operator: String = if operator_st.location.start.line == token.location.start.line
      && operator_st.location.start.column == (token.location.start.column - 1)
      && token.token_type == TokenType::Operator(OperatorType::Equals)
    {
      self.eat();
      operator_st.value.clone()
    } else {
      "".to_string()
    };
    // operator "(?) =" operator not valid (space between operators)
    if pre_operator == "" && operator_st.value != "" {
      return left.into();
    }
    let operator = pre_operator.as_str();
    let right = self.parse_expr()?;
    if operator == "" {
      ast::Node::Assignment(ast::NodeAssignment {
        identifier: left.clone().to_box(),
        value: right.to_box(),
        location: left.get_location(),
        file: left.get_file(),
      })
      .into()
    } else if operator == "=" {
      ast::Node::Binary(ast::NodeBinary {
        operator: "==".to_string(),
        left: left.clone().to_box(),
        right: right.to_box(),
        location: left.get_location(),
        file: left.get_file(),
      })
      .into()
    } else if ASSIGNMENT_MODIFICATOR.contains(&operator) {
      ast::Node::Assignment(ast::NodeAssignment {
        identifier: left.clone().to_box(),
        value: ast::Node::Binary(ast::NodeBinary {
          operator: operator.to_string(),
          left: left.clone().to_box(),
          right: right.to_box(),
          location: left.get_location(),
          file: left.get_file(),
        })
        .to_box(),
        location: left.get_location(),
        file: left.get_file(),
      })
      .into()
    } else if COMPARISON.contains(&operator) {
      ast::Node::Binary(ast::NodeBinary {
        operator: operator.to_string(),
        left: left.clone().to_box(),
        right: right.to_box(),
        location: left.get_location(),
        file: left.get_file(),
      })
      .into()
    } else {
      let line = self.source.lines().nth(token.location.start.line).unwrap();
      Err(ast::NodeError {
        message: "Operador no válido para asignación".to_string(),
        location: token.location,
        meta: format!("{}\0{}\0{}", token.meta, line, token.value),
      })
    }
  }
  fn parse_complex_expr(
    &mut self,
    left: ast::Node,
    operator_st: SemiToken,
  ) -> Result<ast::Node, NodeError> {
    let (left, mut operator_st) = self.parse_back_unary_expr(left.clone(), operator_st)?;
    let token = self.at();
    if token.token_type == TokenType::Error {
      return Err(ast::NodeError {
        message: token.value.clone(),
        location: token.location,
        meta: token.meta,
      });
    }
    if token.token_type != TokenType::Operator(OperatorType::None) && operator_st.value == "" {
      return left.into();
    }
    if token.token_type != TokenType::Operator(OperatorType::None)
      && (operator_st.value == ">" || operator_st.value == "<")
    {
      let operator = operator_st.value;
      let right = self.parse_expr()?;
      return ast::Node::Binary(ast::NodeBinary {
        operator,
        left: left.clone().to_box(),
        right: right.to_box(),
        location: left.get_location(),
        file: left.get_file(),
      })
      .into();
    }
    if operator_st.value == "=" && token.token_type != TokenType::Operator(OperatorType::Equals) {
      operator_st.value = "".to_string();
      return self.parse_assignment_expr(left, operator_st);
    }
    match token.value.as_str() {
      "=" => self.parse_assignment_expr(left, operator_st),
      "?" | "|" | "&" => {
        self.eat();
        if operator_st.location.start.line != token.location.start.line
          || operator_st.location.start.column != (token.location.start.column - 1)
        {
          let line = self.source.lines().nth(token.location.start.line).unwrap();
          return Err(ast::NodeError {
            message: "Se esperaba una expresión (c)".to_string(),
            location: token.location,
            meta: format!("{}\0{}\0{}", token.meta, line, token.value),
          });
        }
        operator_st.value += token.value.as_str();
        operator_st.location.start.column = token.location.start.column;
        if operator_st.value != "??" && operator_st.value != "||" && operator_st.value != "&&" {
          let line = self.source.lines().nth(token.location.start.line).unwrap();
          return Err(ast::NodeError {
            message: "Operador no válido".to_string(),
            location: token.location,
            meta: format!("{}\0{}\0{}", token.meta, line, token.value),
          });
        }
        if self.at().token_type == TokenType::Operator(OperatorType::Equals) {
          return self.parse_assignment_expr(left, operator_st);
        }
        let right = self.parse_expr()?;
        ast::Node::Binary(ast::NodeBinary {
          operator: format!("{}", operator_st.value),
          left: left.clone().to_box(),
          right: right.to_box(),
          location: left.get_location(),
          file: left.get_file(),
        })
        .into()
      }
      _ => left.into(),
    }
  }
  fn parse_back_unary_expr(
    &mut self,
    left: ast::Node,
    mut operator_st: SemiToken,
  ) -> Result<(ast::Node, SemiToken), NodeError> {
    let token = self.at();
    if operator_st.value == "?"
      && !(token.token_type == TokenType::Operator(OperatorType::QuestionMark)
        && operator_st.location.start.line == token.location.start.line
        && operator_st.location.start.column == (token.location.start.column - 1))
    {
      let operator = operator_st.value;
      let new_operator = if let TokenType::Operator(_) = token.token_type {
        self.eat();
        token.value
      } else {
        "".to_string()
      };
      operator_st.value = new_operator;
      operator_st.location.start.column = token.location.start.column;
      operator_st.location.start.line = token.location.start.line;
      let data = ast::Node::UnaryBack(ast::NodeUnary {
        operator,
        operand: left.to_box(),
        location: operator_st.location,
        file: self.file_name.clone(),
      });
      let location = operator_st.location;
      return Ok((
        self.parse_complex_expr(data, operator_st)?,
        SemiToken {
          value: if let TokenType::Operator(_) = self.at().token_type {
            self.eat().value
          } else {
            "".to_string()
          },
          location,
        },
      ));
    }
    if operator_st.value == ""
      && (token.token_type == TokenType::Punctuation(PunctuationType::Dot)
        || token.token_type == TokenType::Punctuation(PunctuationType::CircularBracketOpen)
        || token.token_type == TokenType::Punctuation(PunctuationType::QuadrateBracketOpen)
        || token.token_type == TokenType::Punctuation(PunctuationType::DoubleDot))
    {
      let value = self.parse_call_member_expr(left)?;
      let token = self.at();
      return Ok((
        value,
        SemiToken {
          location: token.location,
          value: if let TokenType::Operator(_) = token.token_type {
            self.eat();
            token.value
          } else {
            "".to_string()
          },
        },
      ));
    }
    Ok((left, operator_st))
  }
  fn parse_call_member_expr(&mut self, object: ast::Node) -> Result<ast::Node, NodeError> {
    let member = self.parse_member_expr(object)?;
    let token = self.at();
    if token.token_type == TokenType::Punctuation(PunctuationType::CircularBracketOpen) {
      return self.parse_call_expr(member);
    }
    member.into()
  }
  fn parse_call_expr(&mut self, callee: ast::Node) -> Result<ast::Node, NodeError> {
    let token = self.eat();
    let mut args = List::new();
    while self.not_eof()
      && !(self.at().token_type == TokenType::Punctuation(PunctuationType::CircularBracketClose))
    {
      let arg = self.parse_expr()?;
      args.push(arg);
      let comma = self.at();
      if comma.token_type == TokenType::Punctuation(PunctuationType::Comma) {
        self.eat();
        continue;
      }
      if comma.token_type == TokenType::Punctuation(PunctuationType::CircularBracketClose) {
        break;
      }

      let line = self.source.lines().nth(comma.location.start.line).unwrap();
      return Err(ast::NodeError {
        message: "Se esperaba una coma".to_string(),
        location: comma.location,
        meta: format!("{}\0{}\0{}", comma.meta, line, comma.value),
      });
    }
    let close_paren = self.expect(
      TokenType::Punctuation(PunctuationType::CircularBracketClose),
      "",
    );
    if close_paren.token_type == TokenType::Error {
      return Err(ast::NodeError {
        message: "Se esperaba un paréntesis de cierre".to_string(),
        location: close_paren.location,
        meta: close_paren.meta,
      });
    }
    let call_expr = ast::Node::Call(ast::NodeCall {
      callee: callee.to_box(),
      arguments: args,
      location: token.location,
      file: token.meta,
    });
    let semi_token = if let TokenType::Operator(_) = self.at().token_type {
      let token = self.eat();
      SemiToken {
        location: token.location,
        value: token.value,
      }
    } else {
      SemiToken {
        location: self.at().location,
        value: "".to_string(),
      }
    };
    self.parse_complex_expr(call_expr, semi_token)
  }
  fn parse_member_expr(&mut self, object: ast::Node) -> Result<ast::Node, NodeError> {
    let mut value = object;
    while self.at().token_type == TokenType::Punctuation(PunctuationType::Dot)
      || self.at().token_type == TokenType::Punctuation(PunctuationType::QuadrateBracketOpen)
      || self.at().token_type == TokenType::Punctuation(PunctuationType::DoubleDot)
    {
      let operator = self.eat();
      let computed = operator.value == "[";
      let instance = operator.value == ":";
      let property = if computed {
        self.parse_expr()
      } else {
        if instance {
          if self.at().token_type != TokenType::Punctuation(PunctuationType::DoubleDot) {
            let line = self
              .source
              .lines()
              .nth(operator.location.start.line)
              .unwrap();
            return Err(ast::NodeError {
              location: operator.location,
              message: "Se esperaba un identificador valido".to_string(),
              meta: format!("{}\0{}\0{}", operator.meta, line, operator.value),
            });
          }
          self.eat();
        }
        self.parse_literal_member_expr()
      }?;
      if computed {
        let close = self.expect(
          TokenType::Punctuation(PunctuationType::QuadrateBracketClose),
          "",
        );
        if close.token_type == TokenType::Error {
          return Err(ast::NodeError {
            location: close.location,
            message: "Se esperaba un corchete de cierre".to_string(),
            meta: close.meta,
          });
        }
      }
      value = ast::Node::Member(ast::NodeMember {
        object: value.clone().to_box(),
        member: property.to_box(),
        computed,
        instance,
        location: value.get_location(),
        file: value.get_file(),
      });
    }
    value.into()
  }
  fn parse_literal_member_expr(&mut self) -> Result<ast::Node, NodeError> {
    let token = self.eat();
    match token.token_type {
      TokenType::Identifier | TokenType::Keyword(_) => ast::Node::Identifier(ast::NodeIdentifier {
        location: token.location,
        file: token.meta,
        name: token.value,
      })
      .into(),
      _ => {
        let line = self.source.lines().nth(token.location.start.line).unwrap();
        Err(ast::NodeError {
          location: token.location,
          message: "Se esperaba un identificador valido".to_string(),
          meta: format!("{}\0{}\0{}", token.meta, line, token.value),
        })
      }
    }
  }
  fn parse_literal_expr(&mut self) -> Result<Result<ast::Node, NodeError>, Token<TokenType>> {
    let token = self.at();
    match token.token_type {
      TokenType::Identifier => ast::Node::Identifier(ast::NodeIdentifier {
        name: self.eat().value,
        location: token.location,
        file: token.meta,
      })
      .into(),
      TokenType::NumberLiteral => ast::Node::Number(ast::NodeNumber {
        base: 10,
        value: self.eat().value,
        location: token.location,
        file: token.meta,
      })
      .into(),
      TokenType::Number => {
        self.eat();
        let data = token.value.split("$").collect::<Vec<_>>()[1];
        let base_value = data.split("~").collect::<Vec<_>>();
        let base = base_value[0].parse::<u8>().unwrap();
        let value = base_value[1].to_string();
        ast::Node::Number(ast::NodeNumber {
          base,
          value,
          location: token.location,
          file: token.meta,
        })
        .into()
      }
      TokenType::Byte => ast::Node::Byte(ast::NodeByte {
        value: u8::from_str_radix(&self.eat().value, 2).expect("no es un byte"),
        location: token.location,
        file: token.meta,
      })
      .into(),
      TokenType::StringLiteral => ast::Node::String(ast::NodeString {
        value: List::from_vec(vec![ast::StringData::Str(self.eat().value)]),
        location: token.location,
        file: token.meta,
      })
      .into(),
      TokenType::String => {
        self.eat();
        let line = self.source.lines().nth(token.location.start.line).unwrap();
        let node = string::complex_string(token, line);
        match node {
          Err(e) => Ok(Err(e)),
          Ok(node) => ast::Node::String(node).into(),
        }
      }
      TokenType::Punctuation(PunctuationType::RegularBracketOpen) => Ok(self.parse_object_expr()),
      TokenType::Punctuation(PunctuationType::CircularBracketOpen) => {
        self.eat();
        let expr = self.parse_expr();
        let close_paren = self.expect(
          TokenType::Punctuation(PunctuationType::CircularBracketClose),
          "",
        );
        if close_paren.token_type == TokenType::Error {
          return Ok(Err(ast::NodeError {
            message: "Se esperaba un paréntesis de cierre".to_string(),
            location: close_paren.location,
            meta: close_paren.meta,
          }));
        }
        return Ok(expr);
      }
      TokenType::Punctuation(PunctuationType::QuadrateBracketOpen) => Ok(self.parse_array_expr()),
      TokenType::Operator(
        OperatorType::Minus
        | OperatorType::Plus
        | OperatorType::Negative
        | OperatorType::Not
        | OperatorType::And
        | OperatorType::QuestionMark,
      ) => {
        let expr = self.parse_literal_expr()?;
        let operand = match expr {
          Ok(node) => node.to_box(),
          Err(e) => return Ok(Err(e)),
        };
        ast::Node::UnaryFront(ast::NodeUnary {
          operator: token.value,
          operand,
          location: token.location,
          file: token.meta,
        })
        .into()
      }
      TokenType::Keyword(
        KeywordsType::While
        | KeywordsType::Do
        | KeywordsType::If
        | KeywordsType::Function
        | KeywordsType::Try
        | KeywordsType::Async,
      ) => Ok(self.parse_keyword_value(false, false, false)),
      TokenType::Keyword(KeywordsType::Lazy) => {
        self.eat();
        let expr = self.parse_expr();

        let expression = match expr {
          Ok(node) => node.to_box(),
          Err(e) => return Ok(Err(e)),
        };
        ast::Node::Lazy(ast::NodeExpressionMedicator {
          expression,
          location: token.location,
          file: token.meta,
        })
        .into()
      }
      TokenType::Keyword(KeywordsType::Await) => {
        self.eat();
        let expr = self.parse_expr();
        let expression = match expr {
          Ok(node) => node.to_box(),
          Err(e) => return Ok(Err(e)),
        };
        ast::Node::Await(ast::NodeExpressionMedicator {
          expression,
          location: token.location,
          file: token.meta,
        })
        .into()
      }
      _ => Err(token),
    }
  }
  fn parse_object_expr(&mut self) -> Result<ast::Node, NodeError> {
    let open_brace = self.eat();
    let mut properties = List::new();

    while self.not_eof()
      && !(self.at().token_type == TokenType::Punctuation(PunctuationType::RegularBracketClose))
    {
      let property = self.parse_object_property();
      if property.is_err() {
        return Err(property.err().unwrap());
      }
      let property = property.ok().unwrap();
      properties.push(property);
      let comma = self.at();
      if comma.token_type == TokenType::Punctuation(PunctuationType::Comma) {
        self.eat();
        continue;
      }
      if comma.token_type == TokenType::Punctuation(PunctuationType::RegularBracketClose) {
        break;
      }
      let line = self.source.lines().nth(comma.location.start.line).unwrap();
      return Err(ast::NodeError {
        message: "Se esperaba una coma".to_string(),
        location: comma.location,
        meta: format!("{}\0{}\0{}", comma.meta, line, comma.value),
      });
    }
    let close_brace = self.expect(
      TokenType::Punctuation(PunctuationType::RegularBracketClose),
      "",
    );
    if close_brace.token_type == TokenType::Error {
      return Err(ast::NodeError {
        message: "Se esperaba una llave de cierre".to_string(),
        location: close_brace.location,
        meta: close_brace.meta,
      });
    }
    ast::Node::Object(ast::NodeObject {
      properties,
      location: open_brace.location,
      file: open_brace.meta,
    })
    .into()
  }
  fn parse_object_property(&mut self) -> Result<ast::NodeProperty, ast::NodeError> {
    let token = self.eat();
    match token.token_type {
      TokenType::StringLiteral => {
        let key = token.value;
        let colon = self.expect(
          TokenType::Punctuation(PunctuationType::DoubleDot),
          "Se esperaba dos puntos",
        );
        if colon.token_type == TokenType::Error {
          return Err(ast::NodeError {
            message: colon.value,
            location: colon.location,
            meta: colon.meta,
          });
        }
        let value = self.parse_expr()?;
        return Ok(ast::NodeProperty::Property(key, value));
      }
      TokenType::Identifier | TokenType::Keyword(_) => {
        let key = &token.value;
        let colon = self.eat();
        if colon.token_type == TokenType::Error {
          return Err(ast::NodeError {
            message: "Se esperaba dos puntos".to_string(),
            location: colon.location,
            meta: colon.meta,
          });
        }
        // the key is a variable name and value is an identifier
        if colon.token_type == TokenType::Punctuation(PunctuationType::Comma)
          || colon.token_type == TokenType::Punctuation(PunctuationType::RegularBracketClose)
        {
          self.index -= 1;
          return Ok(ast::NodeProperty::Property(
            key.clone(),
            ast::Node::Identifier(ast::NodeIdentifier {
              name: token.value,
              location: token.location,
              file: token.meta,
            }),
          ));
        }
        if colon.token_type != TokenType::Punctuation(PunctuationType::DoubleDot) {
          let line = self.source.lines().nth(colon.location.start.line).unwrap();
          return Err(ast::NodeError {
            message: "Se esperaba dos puntos".to_string(),
            location: colon.location,
            meta: format!("{}\0{}\0{}", colon.meta, line, colon.value),
          });
        }
        let value = self.parse_expr()?;
        return Ok(ast::NodeProperty::Property(key.clone(), value));
      }
      TokenType::Punctuation(p) => {
        if p == PunctuationType::QuadrateBracketOpen {
          let expr = self.parse_expr();
          let close_bracket = self.expect(
            TokenType::Punctuation(PunctuationType::QuadrateBracketClose),
            "Se esperaba un corchete de cierre",
          );
          if close_bracket.token_type == TokenType::Error {
            let line = self
              .source
              .lines()
              .nth(close_bracket.location.start.line)
              .unwrap();
            let meta = format!("{}\0{}\0{}", close_bracket.meta, line, &close_bracket.value);
            return Err(ast::NodeError {
              message: close_bracket.value,
              location: close_bracket.location,
              meta,
            });
          }
          let key = expr?;
          let colon = self.expect(
            TokenType::Punctuation(PunctuationType::DoubleDot),
            "Se esperaba dos puntos",
          );
          if colon.token_type == TokenType::Error {
            return Err(ast::NodeError {
              message: colon.value,
              location: colon.location,
              meta: colon.meta,
            });
          }
          let value = self.parse_expr()?;
          return Ok(ast::NodeProperty::Dynamic(key, value));
        }
        if p == PunctuationType::Dot {
          let dot = self.expect(
            TokenType::Punctuation(PunctuationType::Dot),
            "Se esperaba un punto",
          );
          if dot.token_type == TokenType::Error {
            return Err(ast::NodeError {
              message: dot.value,
              location: dot.location,
              meta: dot.meta,
            });
          }
          let data = self.parse_expr()?;
          return Ok(ast::NodeProperty::Iterable(data));
        }
        let line = self.source.lines().nth(token.location.start.line).unwrap();
        return Err(ast::NodeError {
          message: "Se esperaba un clave para la propiedad del objeto".to_string(),
          location: token.location,
          meta: format!("{}\0{}\0{}", token.meta, line, token.value),
        });
      }
      _ => {
        let line = self.source.lines().nth(token.location.start.line).unwrap();
        return Err(ast::NodeError {
          message: "Se esperaba un clave para la propiedad del objeto".to_string(),
          location: token.location,
          meta: format!("{}\0{}\0{}", token.meta, line, token.value),
        });
      }
    }
  }
  fn parse_array_expr(&mut self) -> Result<ast::Node, NodeError> {
    let open_bracket = self.eat();
    let mut elements = List::new();

    while self.not_eof()
      && !(self.at().token_type == TokenType::Punctuation(PunctuationType::QuadrateBracketClose))
    {
      let element = self.parse_array_property();
      if element.is_err() {
        return Err(element.err().unwrap());
      }
      let property = element.ok().unwrap();
      elements.push(property);
      let comma = self.at();
      if comma.token_type == TokenType::Punctuation(PunctuationType::Comma) {
        self.eat();
        continue;
      }
      if comma.token_type == TokenType::Punctuation(PunctuationType::QuadrateBracketClose) {
        break;
      }
      let line = self.source.lines().nth(comma.location.start.line).unwrap();
      return Err(ast::NodeError {
        message: "Se esperaba una coma".to_string(),
        location: comma.location,
        meta: format!("{}\0{}\0{}", comma.meta, line, comma.value),
      });
    }
    let close_brace = self.expect(
      TokenType::Punctuation(PunctuationType::QuadrateBracketClose),
      "Se esperaba un corchete de cierre",
    );
    if close_brace.token_type == TokenType::Error {
      return Err(ast::NodeError {
        message: close_brace.value,
        location: close_brace.location,
        meta: close_brace.meta,
      });
    }
    ast::Node::Array(ast::NodeArray {
      elements,
      location: open_bracket.location,
      file: open_bracket.meta,
    })
    .into()
  }
  fn parse_array_property(&mut self) -> Result<ast::NodeProperty, ast::NodeError> {
    let token = self.at();
    match token.token_type {
      TokenType::Punctuation(p) => {
        if p == PunctuationType::Dot {
          self.eat();
          let dot = self.expect(
            TokenType::Punctuation(PunctuationType::Dot),
            "Se esperaba un punto",
          );
          if dot.token_type == TokenType::Error {
            return Err(ast::NodeError {
              message: dot.value,
              location: dot.location,
              meta: dot.meta,
            });
          }
          let data = self.parse_expr()?;
          Ok(ast::NodeProperty::Iterable(data))
        } else {
          let line = self.source.lines().nth(token.location.start.line).unwrap();
          return Err(ast::NodeError {
            message: "Se esperaba un valor para la lista".to_string(),
            location: token.location,
            meta: format!("{}\0{}\0{}", token.meta, line, token.value),
          });
        }
      }
      _ => {
        let element = self.parse_expr()?;
        Ok(ast::NodeProperty::Indexable(element))
      }
    }
  }
}

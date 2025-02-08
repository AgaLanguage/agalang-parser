pub enum ErrorTypes {
  //FmtError(std::fmt::Error),
  IoError(std::io::Error),
  ErrorError(Box<dyn std::error::Error>),
  StringError(String),
}
#[derive(Clone, PartialEq, Debug)]
pub enum ErrorNames {
  None,
  PathError,
  LexerError,
  SyntaxError,
  CustomError(&'static str),
  EnvironmentError,
  MathError,
  TypeError,
}
impl std::fmt::Display for ErrorNames {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      ErrorNames::None => write!(f, ""),
      ErrorNames::TypeError => write!(f, "Error de tipo"),
      ErrorNames::MathError => write!(f, "Error matemático"),
      ErrorNames::PathError => write!(f, "Error ruta"),
      ErrorNames::LexerError => write!(f, "Error léxico"),
      ErrorNames::SyntaxError => write!(f, "Error sintáctico"),
      ErrorNames::EnvironmentError => write!(f, "Error de entorno"),
      ErrorNames::CustomError(s) => write!(f, "{s}"),
    }
  }
}

const RED_ERROR: &str = "\x1b[1m\x1b[91merror\x1b[39m:\x1b[0m";

pub fn error_type_to_string(err: ErrorTypes) -> String {
  match err {
    //ErrorTypes::FmtError(e) => format!("{}", e),
    ErrorTypes::IoError(e) => format!("{}", e),
    ErrorTypes::ErrorError(e) => format!("{}", e),
    ErrorTypes::StringError(e) => e,
  }
}
pub fn error_to_string(type_err: &ErrorNames, error: ErrorTypes) -> String {
  let error = error_type_to_string(error);
  match type_err {
    ErrorNames::None => error,
    _ => format!("{type_err}: {error}"),
  }
}
pub fn show_error(type_err: &ErrorNames, error: ErrorTypes) {
  let data = error_to_string(type_err, error);
  print_error(data);
}
pub fn show_multiple_errors(type_err: &ErrorNames, errors: Vec<ErrorTypes>) {
  for err in errors {
    show_error(type_err, err);
  }
}

pub fn print_error(data: String) {
  println!("{RED_ERROR} {}", data);
}

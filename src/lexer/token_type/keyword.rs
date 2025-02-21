#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum KeywordsType {
  None, // invalid keyword
  Define,
  Constant,
  Name,
  Throw,
  Function,
  If,
  Else,
  Do,
  While,
  Para,
  Romper,
  Return,
  Continue,
  Class,
  Static,
  Public,
  Extender,
  Try,
  Catch,
  Finally,
  As,
  Export,
  Import,
  Lazy,
  Await,
  Async,
  Console,
}
type KeywordsList = [KeywordsType; 28];
const KEYWORDS: KeywordsList = [
  KeywordsType::None,
  KeywordsType::Define,
  KeywordsType::Constant,
  KeywordsType::Name,
  KeywordsType::Function,
  KeywordsType::If,
  KeywordsType::Else,
  KeywordsType::Do,
  KeywordsType::While,
  KeywordsType::Para,
  KeywordsType::Romper,
  KeywordsType::Return,
  KeywordsType::Continue,
  KeywordsType::Class,
  KeywordsType::Static,
  KeywordsType::Public,
  KeywordsType::Extender,
  KeywordsType::Try,
  KeywordsType::Catch,
  KeywordsType::Finally,
  KeywordsType::Export,
  KeywordsType::Import,
  KeywordsType::As,
  KeywordsType::Throw,
  KeywordsType::Lazy,
  KeywordsType::Await,
  KeywordsType::Async,
  KeywordsType::Console
];
impl KeywordsType {
  pub const fn iter() -> KeywordsList {
    KEYWORDS
  }
  pub const fn as_str(&self) -> &str {
    match self {
      KeywordsType::None => "none",
      KeywordsType::Define => "def",
      KeywordsType::Constant => "const",
      KeywordsType::Name => "nombre",
      KeywordsType::Function => "fn",
      KeywordsType::If => "si",
      KeywordsType::Else => "ent",
      KeywordsType::Do => "haz",
      KeywordsType::While => "mien",
      KeywordsType::Para => "para",
      KeywordsType::Romper => "rom",
      KeywordsType::Return => "ret",
      KeywordsType::Continue => "cont",
      KeywordsType::Class => "clase",
      KeywordsType::Static => "est",
      KeywordsType::Public => "pub",
      KeywordsType::Extender => "extiende",
      KeywordsType::Try => "intenta",
      KeywordsType::Catch => "captura",
      KeywordsType::Finally => "finalmente",
      KeywordsType::Export => "exporta",
      KeywordsType::Import => "importa",
      KeywordsType::As => "como",
      KeywordsType::Throw => "lanza",
      KeywordsType::Lazy => "vago",
      KeywordsType::Await => "espera",
      KeywordsType::Async => "asinc",
      KeywordsType::Console => "csl",
    }
  }
  pub fn to_string(&self) -> String {
    self.as_str().to_string()
  }
}

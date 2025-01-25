#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum KeywordsType {
    None, // invalid keyword
    Define, Constant, Name, Throw,
    Function, If, Else, Do, While,
    Para, Romper,Return, Continue,
    Class, Static,Public,Extender,
    Try, Catch,Finally,As, Export,
    Import, Lazy, Await, Async
}
const KEYWORDS: [KeywordsType; 27] = [
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
    KeywordsType::Async
];
impl KeywordsType {
    pub fn iter() -> [KeywordsType; 27] {
        KEYWORDS
    }
    pub fn as_str(&self) -> &str {
        match self {
            KeywordsType::None => "none",
            KeywordsType::Define => "def",
            KeywordsType::Constant => "const",
            KeywordsType::Name => "nombre",
            KeywordsType::Function => "fn",
            KeywordsType::If => "si",
            KeywordsType::Else => "ent",
            KeywordsType::Do => "hacer",
            KeywordsType::While => "mien",
            KeywordsType::Para => "para",
            KeywordsType::Romper => "rom",
            KeywordsType::Return => "ret",
            KeywordsType::Continue => "cont",
            KeywordsType::Class => "clase",
            KeywordsType::Static => "est",
            KeywordsType::Public => "pub",
            KeywordsType::Extender => "extiende",
            KeywordsType::Try => "intentar",
            KeywordsType::Catch => "capturar",
            KeywordsType::Finally => "finalmente",
            KeywordsType::Export => "exportar",
            KeywordsType::Import => "importar",
            KeywordsType::As => "como",
            KeywordsType::Throw => "lanzar",
            KeywordsType::Lazy => "vago",
            KeywordsType::Await => "esperar",
            KeywordsType::Async => "asinc"
        }
    }
    pub fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}

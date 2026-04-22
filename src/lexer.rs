use once_cell::sync::Lazy;
use std::{collections::HashMap, fmt::Display};
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SimpleToken {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    SemiColon,
    Slash,
    Star,
    Equal,
    Bang,
    BangEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    KeyWord(KeyWordType),
    And,
    Or,
}

impl Display for SimpleToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            SimpleToken::LeftParen => "(",
            SimpleToken::RightParen => ")",
            SimpleToken::LeftBrace => "{",
            SimpleToken::RightBrace => "}",
            SimpleToken::Comma => ",",
            SimpleToken::Dot => ".",
            SimpleToken::Minus => "-",
            SimpleToken::Plus => "+",
            SimpleToken::SemiColon => ";",
            SimpleToken::Slash => "/",
            SimpleToken::Star => "*",
            SimpleToken::Equal => "=",
            SimpleToken::Bang => "!",
            SimpleToken::BangEqual => "!=",
            SimpleToken::Greater => ">",
            SimpleToken::GreaterEqual => ">=",
            SimpleToken::Less => "<",
            SimpleToken::LessEqual => "<=",
            SimpleToken::KeyWord(key_word_type) => return write!(f, "{}", key_word_type),
            SimpleToken::And => "&",
            SimpleToken::Or => "|",
        };
        write!(f, "{}", s)
    }
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum KeyWordType {
    Class,
    Else,
    False,
    Fun,
    For,
    Nil,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    If,
}

impl Display for KeyWordType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            KeyWordType::Class => "class",
            KeyWordType::Else => "else",
            KeyWordType::False => "false",
            KeyWordType::Fun => "fun",
            KeyWordType::For => "for",
            KeyWordType::Nil => "nil",
            KeyWordType::Print => "print",
            KeyWordType::Return => "return",
            KeyWordType::Super => "super",
            KeyWordType::This => "this",
            KeyWordType::True => "true",
            KeyWordType::Var => "var",
            KeyWordType::While => "while",
            KeyWordType::If => "if",
        };
        write!(f, "{}", s)
    }
}
static KEY_WORD_STR: Lazy<HashMap<&'static str, KeyWordType>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("class", KeyWordType::Class);
    m.insert("else", KeyWordType::Else);
    m.insert("false", KeyWordType::False);
    m.insert("fun", KeyWordType::Fun);
    m.insert("for", KeyWordType::For);
    m.insert("nil", KeyWordType::Nil);
    m.insert("print", KeyWordType::Print);
    m.insert("return", KeyWordType::Return);
    m.insert("super", KeyWordType::Super);
    m.insert("this", KeyWordType::This);
    m.insert("true", KeyWordType::True);
    m.insert("var", KeyWordType::Var);
    m.insert("while", KeyWordType::While);
    m.insert("if", KeyWordType::If);
    m
});
#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Single(SimpleToken),
    StringLitteral(String),
    Identifier(String),
    Number(f64),
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Single(simple_token) => write!(f, "{}", simple_token),
            Token::StringLitteral(str) => write!(f, "\"{}\"", str),
            Token::Identifier(c) => write!(f, "{}", c),
            Token::Number(n) => write!(f, "{}", n),
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct LocatedToken {
    pub token: Token,
    pub line: usize,
    pub row: usize,
}

impl Display for LocatedToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.token)
    }
}
impl LocatedToken {
    pub fn new(token: Token, line: usize, row: usize) -> Self {
        Self { token, line, row }
    }
}

#[derive(Clone, PartialEq, Default)]
pub struct TokenVec {
    tokens: Vec<LocatedToken>,
}

impl TokenVec {
    pub fn push(&mut self, token: LocatedToken) {
        self.tokens.push(token);
    }
}

impl Display for TokenVec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, token) in self.tokens.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "{}", token)?;
        }
        Ok(())
    }
}
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
enum State {
    Default,
    BuildBang,
    BuildLess,
    BuildGreater,
    BuildIdentOrKeyword, // May end up being a keyword or an identifier
    BuildNumber,
    BuildString,
    BuildStringEscape,
    Last,
}

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
enum Action {
    None,
    Push(SimpleToken),
    PushAndGoBack(SimpleToken),
    PushString,
    PushNumber,
    PushEscapedInString,
    PushIdentifierOrKeyWord,
    Error,
}

struct Fsm {
    fsm: Vec<Vec<(State, Action)>>,
}

impl Fsm {
    fn transition(&mut self, c: char, input: State, output: (State, Action)) {
        self.fsm[c as usize][input as usize] = output;
    }

    fn transitions(&mut self, characters: &str, input: State, output: (State, Action)) {
        for c in characters.chars() {
            self.fsm[c as usize][input as usize] = output;
        }
    }

    // All characters that are not included in the input character list
    fn transitions_anti(&mut self, characters: &str, input: State, output: (State, Action)) {
        for c in 0..255u8 {
            let c = c as char;
            if !characters.contains(c) {
                self.fsm[c as usize][input as usize] = output;
            }
        }
    }

    pub fn compute(&self, character: u8, state: State) -> (State, Action) {
        self.fsm[character as usize][state as usize]
    }
    pub fn init() -> Self {
        let mut fsm = Vec::with_capacity(u8::MAX as usize);
        fsm.resize(u8::MAX as usize, vec![]);

        for i in 0..u8::MAX {
            let mut n = vec![];
            n.resize(State::Last as usize, (State::Default, Action::Error));
            fsm[i as usize] = n;
        }
        Self { fsm }
    }
    pub fn build(&mut self) {
        use SimpleToken::*;
        use State::*;
        let alphabet_l = "abcdefghijklmnopqrstuvwxyz";
        let alphabet_u = alphabet_l.to_uppercase();
        let both_alpabet = alphabet_l.to_string() + &alphabet_u;
        let digits = "0123456789";
        let alpha_numerical = both_alpabet.clone() + digits;
        let default = Default;

        // Character to skip
        self.transitions(" \t\n", Default, (Default, Action::None));
        // Single Tokens
        let single_token_chars = "(){},.-+*;/&|=";
        let single_token_token = [
            LeftParen, RightParen, LeftBrace, RightBrace, Comma, Dot, Minus, Plus, Star, SemiColon,
            Slash, And, Or, Equal,
        ];
        for (character, token) in single_token_chars.chars().zip(single_token_token) {
            self.transition(character, default, (Default, Action::Push(token)));
        }
        // Token that may be either one character or two, like ! and !=
        // let single_or_dual_token = [Bang, Greater, Less];
        self.transition('!', Default, (BuildBang, Action::None));
        self.transition('=', BuildBang, (Default, Action::Push(BangEqual)));
        self.transition('<', Default, (BuildLess, Action::None));
        self.transition('=', BuildLess, (Default, Action::Push(LessEqual)));
        self.transition('>', Default, (BuildGreater, Action::None));
        self.transition('=', BuildGreater, (Default, Action::Push(GreaterEqual)));

        self.transitions_anti("=", BuildBang, (Default, Action::PushAndGoBack(Bang)));
        self.transitions_anti("=", BuildGreater, (Default, Action::PushAndGoBack(Greater)));
        self.transitions_anti("=", BuildLess, (Default, Action::PushAndGoBack(Less)));

        // String litterals like "Banana"
        self.transition('\"', Default, (BuildString, Action::None));
        self.transition('\\', BuildString, (BuildStringEscape, Action::None));
        self.transitions_anti("\"", BuildString, (BuildString, Action::None));
        self.transitions(
            "\\\"",
            BuildStringEscape,
            (Default, Action::PushEscapedInString),
        );
        self.transition('\"', BuildString, (Default, Action::PushString));

        // Identifier and keywords
        self.transitions(&both_alpabet, Default, (BuildIdentOrKeyword, Action::None));
        self.transition('_', Default, (BuildIdentOrKeyword, Action::None));
        self.transitions(
            &("_".to_string() + &alpha_numerical),
            BuildIdentOrKeyword,
            (BuildIdentOrKeyword, Action::None),
        );
        self.transitions(
            "&|(){},.-+*;/<>! \n",
            BuildIdentOrKeyword,
            (Default, Action::PushIdentifierOrKeyWord),
        );

        // Numbers
        self.transitions(digits, Default, (BuildNumber, Action::None));
        self.transitions(digits, BuildNumber, (BuildNumber, Action::None));
        // self.transition('.', BuildNumber, (BuildNumber, Action::None)); TODO
        self.transitions(
            ")}&|,-+*;/<>! \n",
            BuildNumber,
            (Default, Action::PushNumber),
        );
    }
}
struct Lexer {
    fsm: Fsm,
    source: String,
    source_name: String,
    current: usize,
    start: usize,
    line: usize,
    row: usize,
    state: State,
    pub tokens: TokenVec,
}

impl Lexer {
    pub fn new(source: String, source_name: String) -> Self {
        let source = source;
        let mut fsm = Fsm::init();
        fsm.build();
        Self {
            fsm: fsm,
            source,
            source_name,
            tokens: Default::default(),
            line: 1,
            start: 0,
            current: 0,
            state: State::Default,
            row: 1,
        }
    }

    fn error(&self, message: impl Display) {
        eprintln!(
            "Error: {message} at {}:{}:{}",
            self.source_name, self.line, self.row
        );
    }

    fn is_at_end(&self) -> bool {
        self.source.len() == self.current
    }

    fn advance(&mut self) {
        self.current += 1;
    }

    fn current(&mut self) -> char {
        self.source[self.current..].chars().next().unwrap()
    }

    fn go_back(&mut self) {
        self.current -= 1;
        self.row -= 1;
    }

    fn add_token(&mut self, token: Token) {
        self.tokens
            .push(LocatedToken::new(token, self.line, self.row));
    }

    fn push_string(&mut self) {
        self.add_token(Token::StringLitteral(
            // +1 to remove ""
            self.source[self.start + 1..self.current].to_string(),
        ));
    }
    fn _push_escape_in_string(&mut self) {
        todo!();
    }

    fn push_number(&mut self) {
        let s = &self.source[self.start..self.current].as_bytes();
        let mut base: f64 = 0.0;
        for i in 0..s.len() {
            let c = s[i] - '0' as u8;
            base *= 10.;
            base += c as f64;
        }
        self.add_token(Token::Number(base));
        self.go_back();
    }

    fn push_identifier_or_keyword(&mut self) {
        let s = self.source[self.start..self.current].to_string();
        if let Some(kw) = KEY_WORD_STR.get(s.as_str()) {
            self.add_token(Token::Single(SimpleToken::KeyWord(*kw)));
        } else {
            self.add_token(Token::Identifier(s));
        }
        self.go_back();
    }

    pub fn lex(&mut self) {
        while !self.is_at_end() {
            let c = self.current();
            if c == '\n' {
                self.line += 1;
                self.row = 1;
            }
            let (new_state, action) = self.fsm.compute(c as u8, self.state);
            match action {
                Action::None => (),
                Action::Push(simple_token) => self.add_token(Token::Single(simple_token)),
                Action::PushString => self.push_string(),
                Action::PushNumber => self.push_number(),
                Action::PushEscapedInString => self.push_string(),
                Action::PushIdentifierOrKeyWord => self.push_identifier_or_keyword(),
                Action::Error => {
                    self.error(format!("Error: Unexpected character \"{c}\""));
                    return;
                } // Action::Last => {
                //     if new_state != State::Default {
                //         self.error("Unexpected EOF, please finish with ;");
                //     }
                // }
                Action::PushAndGoBack(simple_token) => {
                    self.add_token(Token::Single(simple_token));
                    self.go_back();
                }
            }
            if [
                State::BuildString,
                State::BuildIdentOrKeyword,
                State::BuildNumber,
            ]
            .contains(&new_state)
                && new_state != self.state
            {
                self.start = self.current;
            }
            self.state = new_state;
            self.row += 1;
            self.advance();
        }
    }
}

pub fn lex(source: String, source_name: String) -> TokenVec {
    let mut lexer = Lexer::new(source, source_name);
    lexer.lex();
    lexer.tokens
}

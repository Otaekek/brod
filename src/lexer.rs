use std::{collections::HashMap, fmt::Display, str::Chars};

#[derive(Copy, Clone, Debug)]
pub enum TokenType<'a> {
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
    Bang,
    BangEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Idenfitier(&'a str),
    String(&'a str),
    Number(f64),
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    Eof,
}

#[derive(Copy, Clone, Debug)]
pub struct Token<'a> {
    pub token_type: TokenType<'a>,
    pub line: usize,
}

impl<'a> Token<'a> {
    pub fn new(token_type: TokenType<'a>, line: usize) -> Self {
        Self { token_type, line }
    }
}

#[derive(Copy, Clone, Debug)]
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
enum Action<'a> {
    None,
    Push(TokenType<'a>),
    PushString,
    PushNumber,
    PushEscapedInString,
    PushIdentifierOrKeyWord,
    Error,
    Last,
}

struct Fsm<'a> {
    fsm: Vec<Vec<(State, Action<'a>)>>,
    state: State,
    token_start: usize,
}

impl<'a> Fsm<'a> {
    fn transision(&mut self, c: char, input: State, output: (State, Action<'a>)) {
        self.fsm[c as usize][input as usize] = output;
    }

    fn transisions(&mut self, characters: &str, input: State, output: (State, Action<'a>)) {
        for c in characters.chars() {
            self.fsm[c as usize][input as usize] = output;
        }
    }

    // All characters that are not included in the input character list
    fn transisions_anti(&mut self, characters: &str, input: State, output: (State, Action<'a>)) {
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

        for i in 0..u8::MAX {
            let mut n = vec![];
            n.resize(State::Last as usize, (State::Default, Action::Error));
            fsm[i as usize] = n;
        }
        Self {
            fsm,
            state: State::Default,
            token_start: 0,
        }
    }
    pub fn build(&mut self) {
        use State::*;
        use TokenType::*;
        let alphabet_l = "abcdefghijklmnopqrstuvwxyz";
        let alphabet_u = alphabet_l.to_uppercase();
        let both_alpabet = alphabet_l.to_string() + &alphabet_u;
        let digits = "0123456789";
        let alpha_numerical = both_alpabet.clone() + digits;
        let default = Default;

        // Character to skip
        self.transisions(" \t", Default, (BuildBang, Action::None));
        // Single Tokens
        let single_token_chars = "(){},.-+*;/&|";
        let single_token_token = [
            LeftParen, RightParen, LeftBrace, RightBrace, Comma, Dot, Minus, Plus, Star, SemiColon,
            Slash, And, Or,
        ];
        for (character, token) in single_token_chars.chars().zip(single_token_token) {
            self.transision(character, default, (Default, Action::Push(token)));
        }
        // Token that may be either one character or two, like ! and !=
        // let single_or_dual_token = [Bang, Greater, Less];
        self.transision('!', Default, (BuildBang, Action::None));
        self.transision('=', BuildBang, (Default, Action::Push(BangEqual)));
        self.transision('<', Default, (BuildLess, Action::None));
        self.transision('=', BuildLess, (Default, Action::Push(LessEqual)));
        self.transision('>', Default, (BuildGreater, Action::None));
        self.transision('=', BuildGreater, (Default, Action::Push(GreaterEqual)));

        self.transisions_anti("=", BuildBang, (Default, Action::Push(Bang)));
        self.transisions_anti("=", BuildGreater, (Default, Action::Push(Greater)));
        self.transisions_anti("=", BuildLess, (Default, Action::Push(Less)));

        // String litterals like "Banana"
        self.transision('\"', Default, (BuildString, Action::None));
        self.transision('\\', BuildString, (BuildStringEscape, Action::None));
        self.transisions_anti("\"", BuildString, (BuildString, Action::PushString));
        self.transisions(
            "\\\"",
            BuildStringEscape,
            (BuildString, Action::PushEscapedInString),
        );
        self.transision('\"', BuildString, (BuildString, Action::None));

        // Identifier and keywords
        self.transisions(&both_alpabet, Default, (BuildIdentOrKeyword, Action::None));
        self.transisions(
            &("_".to_string() + &alpha_numerical),
            BuildIdentOrKeyword,
            (BuildIdentOrKeyword, Action::None),
        );
        self.transisions(
            "&|(){},.-+*;/<>! \n",
            BuildIdentOrKeyword,
            (Default, Action::PushIdentifierOrKeyWord),
        );

        // Numbers
        self.transisions(digits, Default, (BuildNumber, Action::None));
        self.transisions(digits, BuildNumber, (BuildNumber, Action::None));
        self.transision('.', BuildNumber, (BuildNumber, Action::None));
        self.transisions(
            "&|,-+*;/<>! \n",
            BuildNumber,
            (BuildNumber, Action::PushNumber),
        );
    }
}
pub struct Lexer<'a> {
    fsm: Fsm<'a>,
    literals: Vec<String>,
    source: String,
    source_name: String,
    current: usize,
    start: usize,
    line: usize,
    state: State,
    pub tokens: Vec<Token<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: String, source_name: String) -> Self {
        let source = source;
        let mut fsm = Fsm::init();
        fsm.build();
        Self {
            fsm: fsm,
            literals: Vec::with_capacity(4092),
            source,
            source_name,
            tokens: vec![],
            line: 0,
            start: 0,
            current: 0,
            state: State::Default,
        }
    }
    fn error(&self, message: impl Display) {
        eprintln!("Error: line {}: {message}", self.line);
    }
    fn is_at_end(&self) -> bool {
        self.source.len() == self.current
    }
    fn advance(&mut self) -> char {
        self.current += 1;
        self.source[self.current..].chars().next().unwrap()
    }
    fn go_back(&mut self) {
        self.current -= 1;
    }
    fn peak(&mut self) -> Option<char> {
        self.source[self.current..]
            .chars()
            .peekable()
            .peek()
            .cloned()
    }

    fn add_token(&mut self, token_type: TokenType<'a>) {
        self.tokens.push(Token::new(token_type, self.line));
    }

    pub fn lex(&mut self) {
        while !self.is_at_end() {
            let c = self.advance();
            if c == '\n' {
                self.line += 1;
            }
            // let fsm_result = self.fsm[c as u8]
        }
    }
}

// '(' => self.add_token(TokenType::LeftParen),
//          ')' => self.add_token(TokenType::RightParen),
//          '{' => self.add_token(TokenType::LeftBrace),
//          '}' => self.add_token(TokenType::RightBrace),
//          ',' => self.add_token(TokenType::Comma),
//          '.' => self.add_token(TokenType::Dot),
//          '-' => self.add_token(TokenType::Minus),
//          '+' => self.add_token(TokenType::Plus),
//          ';' => self.add_token(TokenType::SemiColon),
//          '/' => self.add_token(TokenType::Slash),
//          '*' => self.add_token(TokenType::Star),
//          '!' => match self.peak() {
//              Some('=') => {
//                  self.add_token(TokenType::BangEqual);
//                  self.advance();
//              }
//              _ => self.add_token(TokenType::Bang),
//          },
//          '>' => match self.peak() {
//              Some('=') => {
//                  self.add_token(TokenType::GreaterEqual);
//                  self.advance();
//              }
//              _ => self.add_token(TokenType::Greater),
//          },
//          '<' => match self.peak() {
//              Some('=') => {
//                  self.add_token(TokenType::LessEqual);
//                  self.advance();
//              }
//              _ => self.add_token(TokenType::Less),
//          },
//          '&' => match self.peak() {
//              Some('&') => {
//                  self.add_token(TokenType::And);
//                  self.advance();
//              }
//              _ => self.error("& is not a valid operator"),
//          },
//          '|' => match self.peak() {
//              Some('|') => {
//                  self.add_token(TokenType::And);
//                  self.advance();
//              }
//              _ => self.error("| is not a valid operator"),
//          },
//          // 'a' => self.add_token(TokenType::Class),
//          // 'a' => self.add_token(TokenType::Else),
//          // 'a' => self.add_token(TokenType::False),
//          // 'a' => self.add_token(TokenType::Fun),
//          // 'a' => self.add_token(TokenType::For),
//          // 'a' => self.add_token(TokenType::If),
//          // 'a' => self.add_token(TokenType::Nil),
//          // 'a' => self.add_token(TokenType::Print),
//          // 'a' => self.add_token(TokenType::Return),
//          // 'a' => self.add_token(TokenType::Super),
//          // 'a' => self.add_token(TokenType::This),
//          // 'a' => self.add_token(TokenType::True),
//          // 'a' => self.add_token(TokenType::Var),
//          // 'a' => self.add_token(TokenType::While),
//          // Idenfitier(&'a str),
//          // String(&'a str),
//          // Number(f64),
//          unknown => self.error(format!("Unknown character: {unknown}")),

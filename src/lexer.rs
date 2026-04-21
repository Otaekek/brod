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

#[derive(Clone, Debug, PartialEq)]
pub struct LocatedToken {
    pub token: Token,
    pub line: usize,
    pub row: usize,
}

impl LocatedToken {
    pub fn new(token: Token, line: usize, row: usize) -> Self {
        Self { token, line, row }
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
    fn transision(&mut self, c: char, input: State, output: (State, Action)) {
        self.fsm[c as usize][input as usize] = output;
    }

    fn transisions(&mut self, characters: &str, input: State, output: (State, Action)) {
        for c in characters.chars() {
            self.fsm[c as usize][input as usize] = output;
        }
    }

    // All characters that are not included in the input character list
    fn transisions_anti(&mut self, characters: &str, input: State, output: (State, Action)) {
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
        self.transisions(" \t", Default, (Default, Action::None));
        // Single Tokens
        let single_token_chars = "(){},.-+*;/&|=";
        let single_token_token = [
            LeftParen, RightParen, LeftBrace, RightBrace, Comma, Dot, Minus, Plus, Star, SemiColon,
            Slash, And, Or, Equal,
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
        self.transisions_anti("\"", BuildString, (BuildString, Action::None));
        self.transisions(
            "\\\"",
            BuildStringEscape,
            (Default, Action::PushEscapedInString),
        );
        self.transision('\"', BuildString, (Default, Action::PushString));

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
        // self.transision('.', BuildNumber, (BuildNumber, Action::None)); TODO
        self.transisions(
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
    pub tokens: Vec<LocatedToken>,
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
            tokens: vec![],
            line: 0,
            start: 0,
            current: 0,
            state: State::Default,
            row: 0,
        }
    }

    fn error(&self, message: impl Display) {
        eprintln!("Error: line {}: {message}", self.line);
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
        // self.go_back();
    }
    fn _push_escape_in_string(&mut self) {
        todo!();
        // self.go_back();
    }

    fn push_number(&mut self) {
        let s = &self.source[self.start..self.current].as_bytes();
        let mut base: f64 = 0.0;
        for i in 0..s.len() {
            let c = s[i] - '0' as u8;
            assert!(c > 0 && c < 9);
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
                // self.advance();
                self.line += 1;
                self.row = 0;
            }
            let (new_state, action) = self.fsm.compute(c as u8, self.state);
            println!("{:#?} {} {:#?}", self.state, c, new_state);
            match action {
                Action::None => (),
                Action::Push(token_type) => self.add_token(Token::Single(token_type)),
                Action::PushString => self.push_string(),
                Action::PushNumber => self.push_number(),
                Action::PushEscapedInString => self.push_string(),
                Action::PushIdentifierOrKeyWord => self.push_identifier_or_keyword(),
                Action::Error => {
                    println!("{:#?}", self.tokens);
                    self.error(format!(
                        "Error: Unexpected character {c} at in {}:{}:{}",
                        self.source_name, self.line, self.row,
                    ));
                    return;
                } // Action::Last => {
                  //     if new_state != State::Default {
                  //         self.error("Unexpected EOF, please finish with ;");
                  //     }
                  // }
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
        println!("{:#?}", self.tokens);
    }
}

pub fn lex(source: String, source_name: String) -> Vec<LocatedToken> {
    let mut lexer = Lexer::new(source, source_name);
    lexer.lex();
    lexer.tokens
}

#[test]
fn lexer_simple() {
    assert_eq!(
        vec![
            LocatedToken::new(
                Token::Single(SimpleToken::KeyWord(KeyWordType::Print)),
                0,
                0
            ),
            LocatedToken::new(
                Token::Single(SimpleToken::KeyWord(KeyWordType::Print)),
                0,
                0
            ),
            LocatedToken::new(
                Token::Single(SimpleToken::KeyWord(KeyWordType::Print)),
                0,
                0
            )
        ],
        lex("print 1;".to_string(), "test".to_string())
    );
}

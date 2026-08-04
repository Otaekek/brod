use crate::diagnostic::{Diagnostic, Span, Stage};
use enum_display::EnumDisplay;
use once_cell::sync::Lazy;
use std::{collections::HashMap, fmt::Display};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SimpleToken {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Dot,
    Minus,
    Plus,
    Colon,
    SemiColon,
    Question,
    Slash,
    // Comment,
    Star,
    Equal,
    EqualEqual,
    Bang,
    BangEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    KeyWord(KeyWordType),
    And,
    Or,
    // Newline,
}

impl Display for SimpleToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            SimpleToken::LeftParen => "(",
            SimpleToken::RightParen => ")",
            SimpleToken::LeftBrace => "{",
            SimpleToken::RightBrace => "}",
            SimpleToken::LeftBracket => "[",
            SimpleToken::RightBracket => "]",
            SimpleToken::Comma => ",",
            SimpleToken::Dot => ".",
            SimpleToken::Minus => "-",
            SimpleToken::Plus => "+",
            SimpleToken::SemiColon => ";",
            SimpleToken::Slash => "/",
            SimpleToken::Star => "*",
            SimpleToken::Equal => "=",
            SimpleToken::EqualEqual => "==",
            SimpleToken::Bang => "!",
            SimpleToken::BangEqual => "!=",
            SimpleToken::Greater => ">",
            SimpleToken::GreaterEqual => ">=",
            SimpleToken::Less => "<",
            SimpleToken::LessEqual => "<=",
            SimpleToken::KeyWord(key_word_type) => return write!(f, "{}", key_word_type),
            SimpleToken::And => "&",
            SimpleToken::Or => "|",
            // SimpleToken::Comment => unreachable!(),
            SimpleToken::Colon => ":",
            SimpleToken::Question => "?",
            // SimpleToken::Newline => "newline",
        };
        write!(f, "{}", s)
    }
}
#[derive(Copy, Clone, Debug, PartialEq, EnumDisplay)]
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
    Elif,
    Enum,
    Break,
    Continue,
    MySelf,
}

static KEY_WORD_STR: Lazy<HashMap<&'static str, KeyWordType>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("class", KeyWordType::Class);
    m.insert("else", KeyWordType::Else);
    m.insert("false", KeyWordType::False);
    m.insert("fn", KeyWordType::Fun);
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
    m.insert("elif", KeyWordType::Elif);
    m.insert("enum", KeyWordType::Enum);
    m.insert("break", KeyWordType::Break);
    m.insert("continue", KeyWordType::Continue);
    m.insert("self", KeyWordType::MySelf);
    m
});

#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
    Single(SimpleToken),
    StringLitteral(String),
    Identifier(String),
    Number(f64),
    Comment(String),
}
impl Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::Single(simple_token) => write!(f, "{}", simple_token),
            TokenKind::StringLitteral(str) => write!(f, "\"{}\"", str),
            TokenKind::Identifier(c) => write!(f, "{}", c),
            TokenKind::Number(n) => write!(f, "{}", n),
            TokenKind::Comment(s) => write!(f, "//{}", s),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}
impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
pub struct TokenVec {
    pub tokens: Vec<Token>,
}

impl TokenVec {
    pub fn push(&mut self, token: Token) {
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
    BuildEqualEqual,
    BuildComment,
    Comment,
    BuildIdentOrKeyword, // May end up being a keyword or an identifier
    BuildNumber,
    BuildNumberWithPoint,
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
    PushComment,
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
        let default = State::Default;

        // Character to skip
        self.transitions(" \t\n", Default, (Default, Action::None));
        // Single Tokens
        let single_token_chars = "(){},.-+*;&|:?";
        let single_token_token = [
            LeftParen, RightParen, LeftBrace, RightBrace, Comma, Dot, Minus, Plus, Star, SemiColon,
            And, Or, Colon, Question,
        ];
        assert!(single_token_token.len() == single_token_chars.len());
        for (character, token) in single_token_chars.chars().zip(single_token_token) {
            self.transition(character, default, (Default, Action::Push(token)));
        }

        // Token that may be either one character or two, like ! and !=
        self.transition('!', Default, (BuildBang, Action::None));
        self.transition('=', BuildBang, (Default, Action::Push(BangEqual)));
        self.transition('<', Default, (BuildLess, Action::None));
        self.transition('=', BuildLess, (Default, Action::Push(LessEqual)));
        self.transition('>', Default, (BuildGreater, Action::None));
        self.transition('=', BuildGreater, (Default, Action::Push(GreaterEqual)));
        self.transition('/', Default, (BuildComment, Action::None));
        self.transition('/', BuildComment, (State::Comment, Action::None));
        self.transition('=', Default, (BuildEqualEqual, Action::None));
        self.transitions("=", BuildEqualEqual, (Default, Action::Push(EqualEqual)));

        self.transitions_anti("=", BuildBang, (Default, Action::PushAndGoBack(Bang)));
        self.transitions_anti(
            "=",
            BuildEqualEqual,
            (Default, Action::PushAndGoBack(Equal)),
        );
        self.transitions_anti("=", BuildGreater, (Default, Action::PushAndGoBack(Greater)));
        self.transitions_anti("=", BuildLess, (Default, Action::PushAndGoBack(Less)));
        self.transitions_anti(
            "/;)}",
            BuildComment,
            (Default, Action::PushAndGoBack(Slash)),
        );

        // Collect comment text until end of line
        self.transitions_anti("", State::Comment, (State::Comment, Action::None));
        self.transition('\n', State::Comment, (Default, Action::PushComment));

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
            "=&|(){},.-+*;/<>!:? \n",
            BuildIdentOrKeyword,
            (Default, Action::PushIdentifierOrKeyWord),
        );

        // Numbers
        self.transitions(digits, Default, (BuildNumber, Action::None));
        self.transitions(digits, BuildNumber, (BuildNumber, Action::None));
        self.transitions(
            digits,
            BuildNumberWithPoint,
            (BuildNumberWithPoint, Action::None),
        );
        self.transition('.', BuildNumber, (BuildNumberWithPoint, Action::None));
        self.transitions(
            "=(){}&|,-+*;/<>!:? \n",
            BuildNumber,
            (Default, Action::PushNumber),
        );
        self.transitions(
            "=(){}&|,-+*;/<>!:? \n",
            BuildNumberWithPoint,
            (Default, Action::PushNumber),
        );
    }
}
struct Lexer {
    fsm: Fsm,
    source: String,
    current: usize,
    start: usize,
    state: State,
    pub tokens: TokenVec,
}

#[derive(Debug, Clone)]
pub struct LexError {
    pub message: String,
    pub span: Span,
}

impl Diagnostic for LexError {
    fn stage(&self) -> Stage {
        Stage::Lexing
    }
    fn span(&self) -> Option<Span> {
        Some(self.span)
    }
    fn message(&self) -> String {
        self.message.clone()
    }
}

impl Lexer {
    pub fn new(source: String) -> Self {
        let source = source;
        let mut fsm = Fsm::init();
        fsm.build();
        Self {
            fsm: fsm,
            source,
            tokens: Default::default(),
            start: 0,
            current: 0,
            state: State::Default,
        }
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

    fn span_inclusive(&self) -> Span {
        Span::new(self.start, self.current + 1)
    }

    fn span_exclusive(&self) -> Span {
        Span::new(self.start, self.current)
    }

    fn add_token(&mut self, kind: TokenKind, span: Span) {
        self.tokens.push(Token::new(kind, span));
    }

    fn push_string(&mut self) {
        let span = self.span_inclusive();
        self.add_token(
            TokenKind::StringLitteral(self.source[self.start + 1..self.current].to_string()),
            span,
        );
    }
    fn _push_escape_in_string(&mut self) {
        todo!();
    }

    fn push_number(&mut self) {
        let s = &self.source[self.start..self.current];
        let number: f64 = s.parse().unwrap();
        let span = self.span_exclusive();
        self.add_token(TokenKind::Number(number), span);
        self.go_back();
    }

    fn push_identifier_or_keyword(&mut self) {
        let s = self.source[self.start..self.current].to_string();
        let span = self.span_exclusive();
        if let Some(kw) = KEY_WORD_STR.get(s.as_str()) {
            self.add_token(TokenKind::Single(SimpleToken::KeyWord(*kw)), span);
        } else {
            self.add_token(TokenKind::Identifier(s), span);
        }
        self.go_back();
    }

    fn push_comment(&mut self) {
        // start points at the second '/', so +1 skips it to get the comment body
        let text = self.source[self.start + 1..self.current].to_string();
        let span = self.span_exclusive();
        self.add_token(TokenKind::Comment(text), span);
    }

    pub fn lex(&mut self) -> Result<(), LexError> {
        self.source.push(' ');
        while !self.is_at_end() {
            if self.state == State::Default {
                self.start = self.current;
            }
            let c = self.current();
            let (new_state, action) = self.fsm.compute(c as u8, self.state);
            match action {
                Action::None => (),
                Action::Push(simple_token) => {
                    let span = self.span_inclusive();
                    self.add_token(TokenKind::Single(simple_token), span);
                }
                Action::PushString => self.push_string(),
                Action::PushNumber => self.push_number(),
                Action::PushEscapedInString => self.push_string(),
                Action::PushIdentifierOrKeyWord => self.push_identifier_or_keyword(),
                Action::PushComment => self.push_comment(),
                Action::Error => {
                    return Err(LexError {
                        message: format!("Unexpected character \"{c}\""),
                        span: Span::point(self.current),
                    });
                } // Action::Last => {
                //     if new_state != State::Default {
                //         self.error("Unexpected EOF, please finish with ;");
                //     }
                // }
                Action::PushAndGoBack(simple_token) => {
                    let span = self.span_exclusive();
                    self.add_token(TokenKind::Single(simple_token), span);
                    self.go_back();
                }
            }
            self.state = new_state;
            self.advance();
        }
        // Flush a comment that ends at EOF with no trailing newline
        if self.state == State::Comment {
            let end = self.source.len() - 1; // exclude the trailing space added above
            let text = self.source[self.start + 1..end].to_string();
            let span = Span::new(self.start, end);
            self.add_token(TokenKind::Comment(text), span);
        }
        Ok(())
    }
}

pub fn lex(source: String) -> Result<TokenVec, LexError> {
    let mut lexer = Lexer::new(source);
    lexer.lex()?;
    Ok(lexer.tokens)
}

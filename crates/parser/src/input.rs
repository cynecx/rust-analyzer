//! See [`Input`].

use crate::SyntaxKind;

use std::collections::VecDeque;

#[allow(non_camel_case_types)]
type bits = u64;

/// Input for the parser -- a sequence of tokens.
///
/// As of now, parser doesn't have access to the *text* of the tokens, and makes
/// decisions based solely on their classification. Unlike `LexerToken`, the
/// `Tokens` doesn't include whitespace and comments. Main input to the parser.
///
/// Struct of arrays internally, but this shouldn't really matter.
#[derive(Default)]
pub struct Input {
    kind: Vec<SyntaxKind>,
    joint: Vec<bits>,
    contextual_kind: Vec<SyntaxKind>,
}

/// `pub` impl used by callers to create `Tokens`.
impl Input {
    #[inline]
    pub fn push(&mut self, kind: SyntaxKind) {
        self.push_impl(kind, SyntaxKind::EOF)
    }
    #[inline]
    pub fn push_ident(&mut self, contextual_kind: SyntaxKind) {
        self.push_impl(SyntaxKind::IDENT, contextual_kind)
    }
    /// Sets jointness for the last token we've pushed.
    ///
    /// This is a separate API rather than an argument to the `push` to make it
    /// convenient both for textual and mbe tokens. With text, you know whether
    /// the *previous* token was joint, with mbe, you know whether the *current*
    /// one is joint. This API allows for styles of usage:
    ///
    /// ```
    /// // In text:
    /// tokens.was_joint(prev_joint);
    /// tokens.push(curr);
    ///
    /// // In MBE:
    /// token.push(curr);
    /// tokens.push(curr_joint)
    /// ```
    #[inline]
    pub fn was_joint(&mut self) {
        let n = self.len() - 1;
        let (idx, b_idx) = self.bit_index(n);
        self.joint[idx] |= 1 << b_idx;
    }
    #[inline]
    fn push_impl(&mut self, kind: SyntaxKind, contextual_kind: SyntaxKind) {
        let idx = self.len();
        if idx % (bits::BITS as usize) == 0 {
            self.joint.push(0);
        }
        self.kind.push(kind);
        self.contextual_kind.push(contextual_kind);
    }
}

/// pub(crate) impl used by the parser to consume `Tokens`.
impl Input {
    pub(crate) fn kind(&self, idx: usize) -> SyntaxKind {
        self.kind.get(idx).copied().unwrap_or(SyntaxKind::EOF)
    }
    pub(crate) fn contextual_kind(&self, idx: usize) -> SyntaxKind {
        self.contextual_kind.get(idx).copied().unwrap_or(SyntaxKind::EOF)
    }
    pub(crate) fn is_joint(&self, n: usize) -> bool {
        let (idx, b_idx) = self.bit_index(n);
        self.joint[idx] & 1 << b_idx != 0
    }
}

impl Input {
    fn bit_index(&self, n: usize) -> (usize, usize) {
        let idx = n / (bits::BITS as usize);
        let b_idx = n % (bits::BITS as usize);
        (idx, b_idx)
    }
    fn len(&self) -> usize {
        self.kind.len()
    }
}

pub struct MutableInput<'a> {
    input: &'a Input,
    injected_tokens: VecDeque<SyntaxKind>,
    pos: usize,
}

impl<'a> MutableInput<'a> {
    pub fn new(input: &'a Input) -> Self {
        Self { input, injected_tokens: VecDeque::new(), pos: 0 }
    }

    pub fn bump(&mut self, n: usize) -> bool {
        let bump_injected = n.min(self.injected_tokens.len());
        let _ = self.injected_tokens.drain(0..bump_injected);
        self.pos += n - bump_injected;
        bump_injected > 0
    }

    pub fn inject(&mut self, token: SyntaxKind) {
        self.injected_tokens.push_back(token);
    }

    pub fn kind(&self, offset: usize) -> SyntaxKind {
        if let Some(&kind) = self.injected_tokens.iter().nth(offset) {
            return kind;
        }
        let offset = offset - self.injected_tokens.len();
        self.input.kind(self.pos + offset)
    }

    pub fn contextual_kind(&self, offset: usize) -> SyntaxKind {
        if self.injected_tokens.iter().nth(offset).is_some() {
            return SyntaxKind::EOF;
        }
        let offset = offset - self.injected_tokens.len();
        self.input.contextual_kind(self.pos + offset)
    }

    pub fn is_joint(&self, offset: usize) -> bool {
        if self.injected_tokens.iter().nth(offset).is_some() {
            return false;
        }
        let offset = offset - self.injected_tokens.len();
        self.input.is_joint(self.pos + offset)
    }
}

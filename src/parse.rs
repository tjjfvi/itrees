use std::collections::HashMap;

use crate::*;
use logos::Logos;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
  LexError,
  UnexpectedEOF,
  ExpectedNumber,
  ExpectedTree,
  InvalidClose,
  ExpectedEq,
}

impl Default for Error {
  fn default() -> Self {
    Error::LexError
  }
}

#[derive(Clone, Copy, Logos, Debug, PartialEq, Eq)]
#[logos(skip r"[ \t\n]+")]
#[logos(error = Error)]
pub enum Token<'a> {
  #[token("=")]
  Eq,
  #[token("*")]
  Era,
  #[token("(")]
  OpenParen,
  #[token(")")]
  CloseParen,
  #[token("[")]
  OpenBracket,
  #[token("]")]
  CloseBracket,
  #[token("{")]
  OpenBrace,
  #[token("}")]
  CloseBrace,

  #[regex("[0-9]+", |lex| lex.slice().parse().ok())]
  Number(usize),

  #[regex(r"[_a-zA-Z][_0-9a-zA-Z]*")]
  Ident(&'a str),
}

fn finish_trees(trees: Vec<PackedNode>) -> *mut [PackedNode] {
  let mut data = trees.into_boxed_slice();
  for word in &mut *data {
    *word = finish_word(*word);
    match word.unpack() {
      Node::Auxiliary(r) => unsafe {
        *r.0 = Node::Auxiliary(Tree(word as *mut _)).pack();
      },
      _ => {}
    }
  }
  Box::into_raw(data)
}

fn finish_tree(tree: Vec<PackedNode>) -> Tree {
  let mut data = tree.into_boxed_slice();
  for word in &mut *data {
    *word = finish_word(*word);
    match word.unpack() {
      Node::Auxiliary(r) => unsafe {
        *r.0 = Node::Auxiliary(Tree(word as *mut _)).pack();
      },
      _ => {}
    }
  }
  Tree(Box::into_raw(data) as *mut _)
}

fn finish_word(word: PackedNode) -> PackedNode {
  match word.unpack() {
    Node::Auxiliary(r) => unsafe { *r.0 },
    _ => word,
  }
}

fn parse_tree_into<'a>(
  into_kind: Option<usize>,
  into: &mut Vec<PackedNode>,
  lexer: &mut impl Iterator<Item = Result<Token<'a>, Error>>,
  scope: &mut HashMap<&'a str, Tree>,
  vars: &mut Vec<*mut (PackedNode, PackedNode)>,
) -> Result<(), Error> {
  let (kind, close) = match lexer.next().ok_or(Error::UnexpectedEOF)?? {
    Token::Ident(n) => {
      into.push(
        Node::Auxiliary(match scope.entry(n) {
          std::collections::hash_map::Entry::Occupied(e) => e.remove(),
          std::collections::hash_map::Entry::Vacant(e) => {
            let mut b = Box::new((PackedNode::ERA, PackedNode::ERA));
            unsafe {
              b.0 = Node::Auxiliary(Tree(&b.1 as *const _ as *mut _)).pack();
              b.1 = Node::Auxiliary(Tree(&b.0 as *const _ as *mut _)).pack();
              let b = Box::into_raw(b);
              vars.push(b);
              e.insert(Tree(&mut (*b).1 as *mut _));
              Tree(&mut (*b).0 as *mut _)
            }
          }
        })
        .pack(),
      );
      return Ok(());
    }
    Token::Era => {
      into.push(PackedNode::ERA);
      return Ok(());
    }
    Token::OpenParen => (0, Token::CloseParen),
    Token::OpenBracket => (1, Token::CloseBracket),
    Token::OpenBrace => match lexer.next().ok_or(Error::UnexpectedEOF)?? {
      Token::Number(n) => (n, Token::CloseBrace),
      _ => Err(Error::ExpectedNumber)?,
    },
    _ => Err(Error::ExpectedTree)?,
  };
  if Some(kind) != into_kind {
    let mut tree = vec![Node::Ctr(kind).pack()];
    parse_tree_into(Some(kind), &mut tree, lexer, scope, vars)?;
    parse_tree_into(Some(kind), &mut tree, lexer, scope, vars)?;
    into.push(Node::Principal(finish_tree(tree)).pack());
  } else {
    into.push(Node::Ctr(kind).pack());
    parse_tree_into(into_kind, into, lexer, scope, vars)?;
    parse_tree_into(into_kind, into, lexer, scope, vars)?;
  }
  if lexer.next().ok_or(Error::UnexpectedEOF)?? != close {
    Err(Error::InvalidClose)?
  }
  Ok(())
}

pub fn parse_program<'a>(source: &'a str) -> Result<(*mut [PackedNode], Net), Error> {
  let mut lexer = Token::lexer(source).peekable();
  let mut trees = vec![];
  let mut net = Net::default();
  let (mut scope, mut vars) = Default::default();
  while !matches!(lexer.peek(), Some(Ok(Token::Eq)) | None) {
    parse_tree_into(None, &mut trees, &mut lexer, &mut scope, &mut vars)?;
  }
  if matches!(lexer.peek(), Some(Ok(Token::Eq))) {
    lexer.next();
    parse_tree_into(None, &mut trees, &mut lexer, &mut scope, &mut vars)?;
    net.link(
      finish_word(trees.pop().unwrap()).unpack(),
      finish_word(trees.pop().unwrap()).unpack(),
    );
  }
  while lexer.peek().is_some() {
    parse_tree_into(None, &mut trees, &mut lexer, &mut scope, &mut vars)?;
    if lexer.next() != Some(Ok(Token::Eq)) {
      Err(Error::ExpectedEq)?
    }
    parse_tree_into(None, &mut trees, &mut lexer, &mut scope, &mut vars)?;
    net.link(
      finish_word(trees.pop().unwrap()).unpack(),
      finish_word(trees.pop().unwrap()).unpack(),
    );
  }
  let trees = finish_trees(trees);
  net.active.reverse();
  Ok((trees, net))
}

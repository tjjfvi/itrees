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
      Node::Ref(Ref::Auxiliary(r)) => unsafe {
        *r = Ref::Auxiliary(word as *mut _ as *mut _).pack();
      },
      _ => {}
    }
  }
  Box::into_raw(data)
}

fn finish_tree(tree: Vec<PackedNode>) -> OwnedTree {
  let mut data = tree.into_boxed_slice();
  for word in &mut data[1..] {
    *word = finish_word(*word);
    match word.unpack() {
      Node::Ref(Ref::Auxiliary(r)) => unsafe {
        *r = Ref::Auxiliary(word as *mut _ as *mut _).pack();
      },
      _ => {}
    }
  }
  OwnedTree(Box::into_raw(data) as *mut _)
}

fn finish_word(word: PackedNode) -> PackedNode {
  match word.unpack() {
    Node::Ref(Ref::Auxiliary(r)) => unsafe { PackedNode((*r).0) },
    _ => word,
  }
}

fn parse_tree_into<'a>(
  into_kind: Option<usize>,
  into: &mut Vec<PackedNode>,
  lexer: &mut impl Iterator<Item = Result<Token<'a>, Error>>,
  scope: &mut HashMap<&'a str, *mut PackedRef>,
  vars: &mut Vec<*mut (PackedRef, PackedRef)>,
) -> Result<(), Error> {
  let (kind, close) = match lexer.next().ok_or(Error::UnexpectedEOF)?? {
    Token::Ident(n) => {
      into.push(
        Node::Ref(Ref::Auxiliary(match scope.entry(n) {
          std::collections::hash_map::Entry::Occupied(e) => e.remove(),
          std::collections::hash_map::Entry::Vacant(e) => {
            let mut b = Box::new((PackedRef::NULL, PackedRef::NULL));
            unsafe {
              b.0 = Ref::Auxiliary(&b.1 as *const _ as *mut _).pack();
              b.1 = Ref::Auxiliary(&b.0 as *const _ as *mut _).pack();
              let b = Box::into_raw(b);
              vars.push(b);
              e.insert(&mut (*b).1 as *mut _);
              &mut (*b).0 as *mut _
            }
          }
        }))
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
    let mut tree = vec![PackedNode(kind)];
    tree.push(PackedNode::ERA);
    parse_tree_into(Some(kind), &mut tree, lexer, scope, vars)?;
    parse_tree_into(Some(kind), &mut tree, lexer, scope, vars)?;
    tree[1] = Node::Ctr(tree.len() - 1).pack();
    into.push(Node::Ref(Ref::Principal(finish_tree(tree))).pack());
  } else {
    let i = into.len();
    into.push(PackedNode::ERA);
    parse_tree_into(into_kind, into, lexer, scope, vars)?;
    parse_tree_into(into_kind, into, lexer, scope, vars)?;
    into[i] = Node::Ctr(into.len() - i).pack();
  }
  if lexer.next().ok_or(Error::UnexpectedEOF)?? != close {
    Err(Error::InvalidClose)?
  }
  Ok(())
}

pub fn parse_program<'a>(
  lexer: &mut impl Iterator<Item = Result<Token<'a>, Error>>,
) -> Result<(*mut [PackedNode], Net), Error> {
  let mut lexer = lexer.peekable();
  let mut trees = vec![];
  let mut net = Net::default();
  let (mut scope, mut vars) = Default::default();
  while !matches!(lexer.peek(), Some(Ok(Token::Eq)) | None) {
    parse_tree_into(None, &mut trees, &mut lexer, &mut scope, &mut vars)?;
  }
  if matches!(lexer.peek(), Some(Ok(Token::Eq))) {
    lexer.next();
    parse_tree_into(None, &mut trees, &mut lexer, &mut scope, &mut vars)?;
    match (
      finish_word(trees.pop().unwrap()).unpack(),
      finish_word(trees.pop().unwrap()).unpack(),
    ) {
      (Node::Era, Node::Era) => {}
      (Node::Ctr(_), _) | (_, Node::Ctr(_)) => unreachable!(),
      (Node::Era, Node::Ref(r)) => net.erase(r),
      (Node::Ref(r), Node::Era) => net.erase(r),
      (Node::Ref(a), Node::Ref(b)) => net.link(b, a),
    }
  }
  while lexer.peek().is_some() {
    parse_tree_into(None, &mut trees, &mut lexer, &mut scope, &mut vars)?;
    if lexer.next() != Some(Ok(Token::Eq)) {
      Err(Error::ExpectedEq)?
    }
    parse_tree_into(None, &mut trees, &mut lexer, &mut scope, &mut vars)?;
    match (
      finish_word(trees.pop().unwrap()).unpack(),
      finish_word(trees.pop().unwrap()).unpack(),
    ) {
      (Node::Era, Node::Era) => {}
      (Node::Ctr(_), _) | (_, Node::Ctr(_)) => unreachable!(),
      (Node::Era, Node::Ref(r)) => net.erase(r),
      (Node::Ref(r), Node::Era) => net.erase(r),
      (Node::Ref(a), Node::Ref(b)) => net.link(b, a),
    }
  }
  let trees = finish_trees(trees);
  net.active.reverse();
  Ok((trees, net))
}

use crate::*;

fn print_tree(
  f: &mut std::fmt::Formatter,
  kind: Option<usize>,
  tree: Tree,
) -> Result<Tree, std::fmt::Error> {
  match tree.node() {
    Node::Era => {
      write!(f, "*")?;
      Ok(tree.offset(1))
    }
    Node::Principal(t) => {
      if Some(t.kind()) == kind {
        write!(f, "#")?;
      }
      print_tree(f, Some(t.kind()), t)?;
      Ok(tree.offset(1))
    }
    Node::Auxiliary(r) => unsafe {
      let (a, b) = (r.0, (*r.0).0 as *mut PackedNode);
      let (a, b) = if (b as usize) < a as usize {
        (b, a)
      } else {
        (a, b)
      };
      write!(f, "{:?}-{:?}", a, b)?;
      Ok(tree.offset(1))
    },
    Node::Ctr(..) => {
      match kind {
        Some(0) => write!(f, "(")?,
        Some(1) => write!(f, "[")?,
        Some(n) => write!(f, "{{{} ", n)?,
        None => write!(f, "{{?? ")?,
      }
      let tree = print_tree(f, kind, tree.offset(1))?;
      write!(f, " ")?;
      let tree = print_tree(f, kind, tree)?;
      match kind {
        Some(0) => write!(f, ")"),
        Some(1) => write!(f, "]"),
        _ => write!(f, "}}"),
      }?;
      Ok(tree)
    }
  }
}

pub struct PrintNet<'a>(pub(crate) *mut [PackedNode], pub(crate) &'a Net);

impl<'a> Debug for PrintNet<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    for &tree in unsafe { &*self.0 } {
      print_tree(f, None, Tree(&mut { tree } as *mut _))?;
      write!(f, "\n")?;
    }
    for &(a, b) in self.1.active.iter().rev() {
      print_tree(f, None, Tree(&mut Node::Principal(a).pack() as *mut _))?;
      write!(f, " = ")?;
      print_tree(f, None, Tree(&mut Node::Principal(b).pack() as *mut _))?;
      write!(f, "\n")?;
    }
    Ok(())
  }
}

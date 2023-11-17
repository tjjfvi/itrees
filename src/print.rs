use crate::*;

fn print_tree(f: &mut std::fmt::Formatter, kind: Option<usize>, tree: Tree) -> std::fmt::Result {
  match tree.root() {
    Node::Era => write!(f, "*"),
    Node::Ref(r) => match r {
      Ref::Principal(t) => {
        if Some(t.kind()) == kind {
          write!(f, "#")?;
        }
        print_tree(f, Some(t.kind()), t.tree())
      }
      Ref::Auxiliary(r) => unsafe {
        let (a, b) = (r, (*r).0 as *mut PackedRef);
        let (a, b) = if (b as usize) < a as usize {
          (b, a)
        } else {
          (a, b)
        };
        write!(f, "{:?}-{:?}", a, b)
      },
    },
    Node::Ctr(_) => {
      match kind {
        Some(0) => write!(f, "(")?,
        Some(1) => write!(f, "[")?,
        Some(n) => write!(f, "{{{} ", n)?,
        None => write!(f, "{{?? ")?,
      }
      print_tree(f, kind, tree.offset(1))?;
      write!(f, " ")?;
      print_tree(f, kind, tree.offset(1 + tree.node(1).length()))?;
      match kind {
        Some(0) => write!(f, ")"),
        Some(1) => write!(f, "]"),
        _ => write!(f, "}}"),
      }
    }
  }
}

pub struct PrintNet<'a>(pub &'a [PackedNode], pub &'a Net);

impl<'a> Debug for PrintNet<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    for &tree in self.0 {
      print_tree(f, None, Tree(&mut { tree } as *mut _))?;
      write!(f, "\n")?;
    }
    for &(a, b) in self.1.active.iter().rev() {
      print_tree(
        f,
        None,
        Tree(&mut Node::Ref(Ref::Principal(a)).pack() as *mut _),
      )?;
      write!(f, " = ")?;
      print_tree(
        f,
        None,
        Tree(&mut Node::Ref(Ref::Principal(b)).pack() as *mut _),
      )?;
      write!(f, "\n")?;
    }
    Ok(())
  }
}

use crate::*;

fn print_tree(f: &mut std::fmt::Formatter, kind: Option<usize>, tree: Tree) -> std::fmt::Result {
  match tree.root().unpack() {
    UnpackedWord::Era => write!(f, "*"),
    UnpackedWord::Ref(r) => match r.unpack() {
      UnpackedRef::Principal(t) => {
        if Some(t.kind()) == kind {
          write!(f, "#")?;
        }
        print_tree(f, Some(t.kind()), t.tree())
      }
      UnpackedRef::Auxiliary(r) => unsafe {
        let (a, b) = (r, (*r).0 as *mut Ref);
        let (a, b) = if (b as usize) < a as usize {
          (b, a)
        } else {
          (a, b)
        };
        write!(f, "{:?}-{:?}", a, b)
      },
    },
    UnpackedWord::Ctr(_) => {
      match kind {
        Some(0) => write!(f, "(")?,
        Some(1) => write!(f, "[")?,
        Some(n) => write!(f, "{{{} ", n)?,
        None => write!(f, "{{?? ")?,
      }
      print_tree(f, kind, tree.offset(1))?;
      write!(f, " ")?;
      print_tree(f, kind, tree.offset(1 + tree.node(1).unpack().length()))?;
      match kind {
        Some(0) => write!(f, ")"),
        Some(1) => write!(f, "]"),
        _ => write!(f, "}}"),
      }
    }
  }
}

pub struct PrintNet<'a>(pub &'a [Word], pub &'a Net);

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
        Tree(&mut UnpackedWord::Ref(UnpackedRef::Principal(a).pack()).pack() as *mut _),
      )?;
      write!(f, " = ")?;
      print_tree(
        f,
        None,
        Tree(&mut UnpackedWord::Ref(UnpackedRef::Principal(b).pack()).pack() as *mut _),
      )?;
      write!(f, "\n")?;
    }
    Ok(())
  }
}

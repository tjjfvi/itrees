#[macro_export]
macro_rules! delegate_debug {
  ({$($impl_line:tt)*} ($self:tt) => $delegatee:expr) => {
    $($impl_line)* {
      fn fmt(&$self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        $delegatee.fmt(f)
      }
    }
  };
}

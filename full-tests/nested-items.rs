pub mod foo {
    /// ~MINIMIZE-ROOT good
    pub fn good(){}
    /// ~REQUIRE-DELETED bad
    pub fn bad(){}
}
struct S;
impl S {
  fn thing_keep(){}
  /// ~REQUIRE-DELETED thing_remove
  fn thing_remove(){}
}
/// ~MINIMIZE-ROOT main
fn main(){
    /// ~MINIMIZE-ROOT S::thing_keep
    S::thing_keep();
}

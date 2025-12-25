pub trait Trait {
    fn method_trait(&self);
}

struct S ;
impl S {
    /// ~REQUIRE-DELETED method_a
    fn method_a(&self) { println!("hello") }
}
impl Trait for S {
    /// ~REQUIRE-DELETED method_trait
    fn method_trait(&self) {
        println!("hello from trait!");
    }
}

/// ~MINIMIZE-ROOT main
fn main(){
    let s = S;
    /// ~MINIMIZE-ROOT s::method_trait
    s.method_trait();
}

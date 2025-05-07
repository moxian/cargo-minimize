use quote::ToTokens;
use syn::{parse_quote, visit_mut::VisitMut};

use crate::processor::{Pass, PassController, ProcessState, SourceFile, tracking};

struct Visitor<'a> {
    current_path: Vec<String>,
    checker: &'a mut PassController,

    loop_expr: syn::Block,
    process_state: ProcessState,
}

impl<'a> Visitor<'a> {
    fn new(checker: &'a mut PassController, use_panics: bool) -> Self {
        let loop_expr = match use_panics {
            false => parse_quote! { { loop {} } },
            true => parse_quote! { { panic!("minimization") } }
        };
        Self {
            current_path: Vec::new(),
            checker,
            process_state: ProcessState::NoChange,
            loop_expr,
        }
    }
}

impl VisitMut for Visitor<'_> {
    fn visit_block_mut(&mut self, block: &mut syn::Block) {
        // if block.stmts.as_slice().is_empty() {
        //     return
        // }
        // if block == &self.loop_expr{
        //     return
        // }
        // // .. else
        // if self.checker.can_process(&self.current_path) {
        //         *block = self.loop_expr.clone();
        //         self.process_state = ProcessState::Changed;
        // } 
        match block.stmts.as_slice() {
            [
                syn::Stmt::Expr(
                    syn::Expr::Loop(syn::ExprLoop {
                        body: loop_body, ..
                    }),
                    _semi,
                ),
            ] if loop_body.stmts.is_empty() => {},
            [
                syn::Stmt::Expr(
                    syn::Expr::Macro(syn::ExprMacro {
                        mac: syn::Macro{
                            path,
                            ..    
                        }, ..
                    }),
                    _semi
                )
            ] if ["unreachable", "panic", "todo"].contains(&path.to_token_stream().to_string().as_str()) => {},
            // Empty bodies are empty already, no need to loopify them.
            [] => {}
            _ if self.checker.can_process(&self.current_path) => {
                *block = self.loop_expr.clone();
                self.process_state = ProcessState::Changed;
            }
            _ => {}
        }
    }

    tracking!();
}


pub struct EverybodyLoops{
    use_panics: bool,
}
impl EverybodyLoops{
    pub fn new(use_panics: bool) -> Self{
        Self{use_panics}
    }
}

impl Pass for EverybodyLoops {
    fn process_file(
        &mut self,
        krate: &mut syn::File,
        _: &SourceFile,
        checker: &mut PassController,
    ) -> ProcessState {
        let mut visitor = Visitor::new(checker, self.use_panics);
        visitor.visit_file_mut(krate);
        visitor.process_state
    }

    fn name(&self) -> &'static str {
        "everybody-loops"
    }
}

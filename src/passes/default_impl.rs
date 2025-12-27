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
            true => parse_quote! { { panic!("minimization") } },
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
    fn visit_trait_item_fn_mut(&mut self, item: &mut syn::TraitItemFn) {
        self.current_path.push(item.sig.ident.to_string());
        syn::visit_mut::visit_trait_item_fn_mut(self, item);
        match item.default.as_mut() {
            None => {
                if self.checker.can_process(&self.current_path) {
                    item.default = Some(self.loop_expr.clone());
                    self.process_state = ProcessState::Changed;
                }
            }
            Some(_) => {},
        };
        self.current_path.pop();
    }

    // tracking!();
    tracking!(visit_item_fn_mut);
    tracking!(visit_impl_item_fn_mut);
    tracking!(visit_item_impl_mut);
    tracking!(visit_item_mod_mut);
    tracking!(visit_field_mut);
    tracking!(visit_item_struct_mut);
    tracking!(visit_item_trait_mut);
}


pub struct DefaultImpl {
    use_panics: bool,
}
impl DefaultImpl {
    pub fn new(use_panics: bool) -> Self {
        Self { use_panics }
    }
}

impl Pass for DefaultImpl {
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
        "default-impl"
    }
}

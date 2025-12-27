
use crate::processor::{Pass, PassController, ProcessState, SourceFile, tracking};
use quote::ToTokens;

use syn::{visit_mut::VisitMut, TraitItemType, TypeParamBound, WherePredicate};

struct Visitor<'a> {
    process_state: ProcessState,
    current_path: Vec<String>,
    checker: &'a mut PassController,
}

impl<'a> Visitor<'a> {
    fn new(checker: &'a mut PassController) -> Self {
        Self {
            process_state: ProcessState::NoChange,
            current_path: Vec::new(),
            checker,
        }
    }

    fn should_retain_where_predicate(&mut self, pred: &WherePredicate) -> bool{
        let mut should_keep = true;
        match pred {
            WherePredicate::Lifetime(_ ) => {},
            WherePredicate::Type(t)=>{
                self.current_path.push(t.bounded_ty.to_token_stream().to_string());
                if self.checker.can_process(&self.current_path){
                    should_keep = false;
                    self.process_state = ProcessState::Changed;
                }
                self.current_path.pop();
            }
            _ => {},
        }
        should_keep
    }

    fn should_retain_bound(&mut self, bound: &TypeParamBound) -> bool{
        let mut should_retain = true;
        match bound{
            TypeParamBound::Trait(trait_bound) => {
                self.current_path.push(trait_bound.path.to_token_stream().to_string());
                if self.checker.can_process(&self.current_path){
                    should_retain = false;
                    self.process_state = ProcessState::Changed;
                }
                self.current_path.pop();
            },
            TypeParamBound::Lifetime(lt) => {
                self.current_path.push(format!("'{}", lt.ident));
                if self.checker.can_process(&self.current_path){
                    should_retain = false;
                    self.process_state = ProcessState::Changed;
                }
                self.current_path.pop();
             }
             , TypeParamBound::PreciseCapture(_) |TypeParamBound::Verbatim(_) => {},
            _ => {}
        }
        should_retain
    }
}


impl VisitMut for Visitor<'_> {
    fn visit_where_clause_mut(&mut self, w: &mut syn::WhereClause){
        self.current_path.push("{{where}}".to_string());
        syn::visit_mut::visit_where_clause_mut(self, w);
        
        w.predicates = w.predicates.iter().filter(|pred| {
            self.should_retain_where_predicate(pred)
        }).cloned().collect();

        self.current_path.pop();
    }
    fn visit_predicate_type_mut(&mut self, pred: &mut syn::PredicateType){
        self.current_path.push(pred.bounded_ty.to_token_stream().to_string());

        pred.bounds = pred.bounds.iter().filter(|bound| 
            self.should_retain_bound(bound)
        ).cloned().collect();

        self.current_path.pop();
    }


    fn visit_trait_item_type_mut(&mut self, tt: &mut TraitItemType){
        self.current_path.push(tt.ident.to_string());
        
        let old_empty = tt.bounds.is_empty();
        tt.bounds = tt.bounds.iter().filter(|bound| 
            self.should_retain_bound(bound)
        ).cloned().collect();
        let new_empty = tt.bounds.is_empty();
        // cleanup when we remove the last bound
        if !old_empty && new_empty{
            assert!(self.process_state == ProcessState::Changed);
            tt.colon_token = None;
        }

        self.current_path.pop();
    }

     tracking!();
    // tracking!(visit_item_fn_mut);
    // tracking!(visit_impl_item_fn_mut);
    // tracking!(visit_item_impl_mut);
    // tracking!(visit_field_mut);
    // tracking!(visit_item_struct_mut);
    // tracking!(visit_item_trait_mut);
}

#[derive(Default)]
pub struct RelaxBounds {}

impl Pass for RelaxBounds {
    fn process_file(
        &mut self,
        krate: &mut syn::File,
        _: &SourceFile,
        checker: &mut PassController,
    ) -> ProcessState {
        let mut visitor = Visitor::new(checker);
        visitor.visit_file_mut(krate);
        visitor.process_state
    }

    fn name(&self) -> &'static str {
        "relax-bounds"
    }
}

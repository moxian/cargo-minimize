use quote::ToTokens;
use syn::{Generics, visit_mut::VisitMut};

use crate::processor::{Pass, PassController, ProcessState, SourceFile, tracking};

struct Visitor<'a> {
    current_path: Vec<String>,
    checker: &'a mut PassController,
    process_state: ProcessState,

    empty_where: syn::WhereClause,
}

impl<'a> Visitor<'a> {
    fn new(checker: &'a mut PassController) -> Self {
        Self {
            current_path: Vec::new(),
            checker,
            process_state: ProcessState::NoChange,

            empty_where: syn::parse_quote! { where },
        }
    }
    fn move_to_where(&mut self, generics: &mut Generics){
        self.current_path.push("<>".to_string());

        for param in &mut generics.params {
            match param {
                syn::GenericParam::Lifetime(lt) => {
                    self.current_path.push(lt.into_token_stream().to_string());
                    if lt.colon_token.is_none() {
                        continue;
                    }
                    if self.checker.can_process(&self.current_path) {
                        let colon = lt
                            .colon_token
                            .take()
                            .expect("we just checked that colon exists...");
                        let bounds = std::mem::take(&mut lt.bounds);

                        let wc = generics
                            .where_clause
                            .get_or_insert_with(|| self.empty_where.clone());
                        wc.predicates
                            .push(syn::WherePredicate::Lifetime(syn::PredicateLifetime {
                                lifetime: lt.lifetime.clone(),
                                colon_token: colon,
                                bounds: bounds,
                            }));

                        self.process_state = ProcessState::Changed;
                    }
                    self.current_path.pop();
                }
                syn::GenericParam::Type(ty) => {
                    self.current_path.push(ty.into_token_stream().to_string());
                    if ty.colon_token.is_none() {
                        continue;
                    }
                    if self.checker.can_process(&self.current_path) {
                        let colon = ty
                            .colon_token
                            .take()
                            .expect("we just checked that colon exists...");
                        let bounds = std::mem::take(&mut ty.bounds);

                        let wc = generics
                            .where_clause
                            .get_or_insert_with(|| self.empty_where.clone());
                        wc.predicates
                            .push(syn::WherePredicate::Type(syn::PredicateType {
                                bounded_ty: syn::Type::Path(syn::TypePath {
                                    path: ty.ident.clone().into(),
                                    qself: None,
                                }),
                                colon_token: colon,
                                lifetimes: None,
                                bounds: bounds,
                            }));

                        self.process_state = ProcessState::Changed;
                    }
                    self.current_path.pop();
                }
                syn::GenericParam::Const(..) => {
                    // can't express `<const FOO: Type>` bounds in `where` clauses. Do nothing.
                }
            }
        }

        self.current_path.pop();
    }

    fn trim_empty_where(&mut self, generics: &mut Generics){
        if let Some(wc) = &generics.where_clause{
            if wc.predicates.is_empty(){
                self.current_path.push("{{where}}".to_string());
                if self.checker.can_process(&self.current_path){
                    generics.where_clause = None;
                    self.process_state = ProcessState::Changed;
                }
                self.current_path.pop();
            }
        }
    }

}

impl VisitMut for Visitor<'_> {
    fn visit_generics_mut(&mut self, generics: &mut Generics) {
       self.move_to_where(generics);
       self.trim_empty_where(generics);
    }
    tracking!();
}

#[derive(Default)]
pub struct CanonicalizeWhere;

impl Pass for CanonicalizeWhere {
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
        "canonicalize-where"
    }
}

//! Module containing flat program optimization
//!
//! @file mod.rs
//! @author Thibaut Schaeffer <thibaut@schaeff.fr>
//! @date 2018

mod canonicalizer;
mod directive;
mod duplicate;
mod redefinition;
mod tautology;

use self::canonicalizer::Canonicalizer;
use self::directive::DirectiveOptimizer;
use self::duplicate::DuplicateOptimizer;
use self::redefinition::RedefinitionOptimizer;
use self::tautology::TautologyOptimizer;

use crate::ir::{ProgIterator, Statement};
use fallible_iterator::{FallibleIterator, IntoFallibleIterator};
use zokrates_field::Field;

impl<T: Field, I: IntoFallibleIterator<Item = Statement<T>, Error = ()>> ProgIterator<T, I> {
    pub fn optimize(
        self,
    ) -> ProgIterator<T, impl IntoFallibleIterator<Item = Statement<T>, Error = ()>> {
        // remove redefinitions
        log::debug!(
            "Optimizer: Remove redefinitions and tautologies and directives and duplicates"
        );

        // define all optimizer steps
        let mut redefinition_optimizer = RedefinitionOptimizer::init(&self);
        let mut tautologies_optimizer = TautologyOptimizer::default();
        let mut directive_optimizer = DirectiveOptimizer::default();
        let mut canonicalizer = Canonicalizer::default();
        let mut duplicate_optimizer = DuplicateOptimizer::default();

        use crate::ir::folder::Folder;

        let r = ProgIterator::<T, _> {
            arguments: self
                .arguments
                .into_iter()
                .map(|a| redefinition_optimizer.fold_argument(a))
                .map(|a| {
                    <TautologyOptimizer as Folder<T>>::fold_argument(&mut tautologies_optimizer, a)
                })
                .map(|a| directive_optimizer.fold_argument(a))
                .map(|a| {
                    <DuplicateOptimizer as Folder<T>>::fold_argument(&mut duplicate_optimizer, a)
                })
                .collect(),
            statements: self
                .statements
                .into_fallible_iter()
                .flat_map(move |s: Statement<T>| redefinition_optimizer.fold_statement(s))
                .flat_map(move |s: Statement<T>| tautologies_optimizer.fold_statement(s))
                .flat_map(move |s: Statement<T>| canonicalizer.fold_statement(s))
                .flat_map(move |s: Statement<T>| directive_optimizer.fold_statement(s))
                .flat_map(move |s: Statement<T>| duplicate_optimizer.fold_statement(s)),
            return_count: self.return_count,
        };

        log::debug!("Done");
        r
    }
}

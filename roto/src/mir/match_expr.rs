//! Lowering a match expression

use super::{TypedValue, TypedVar};
use crate::{
    ast, ice,
    label::LabelRef,
    mir::{Place, Value},
    types::{EnumVariant, Type},
    var::Var,
};
use std::collections::{HashMap, HashSet};

impl super::Lowerer<'_> {
    /// Lower a match expression
    ///
    /// Lowering a match expression is quite tricky. Here's how we do it at
    /// the moment. Take this match expression:
    ///
    /// ```roto
    /// match x {
    ///     A(y) -> {}
    ///     B -> {}
    /// }
    /// ```
    ///
    /// The easiest thing to do is to treat it as one big if-else chain
    /// where we check whether each variant matches, but that is not
    /// particularly efficient. Instead, we should switch directly on the
    /// discriminant.
    ///
    /// First we evaluate `x` and then we go to a `switch` instruction,
    /// which has a label for each pattern. Simple enough. However, there
    /// are 2 things that make it more complicated: default patterns and
    /// guards.
    ///
    /// A guard requires us to do checks after switching on the
    /// discriminant. This expression for example:
    ///
    /// ```roto
    /// match x {
    ///     A(y) | y == 1 -> b1,
    ///     A(y) | y == 2 -> b2,
    ///     B -> b3,
    ///     A(y) -> b4,
    /// }
    /// ```
    ///
    /// Can be compiled like this expression:
    ///
    /// ```roto
    /// match x {
    ///     A(y) -> {
    ///         if y == 1 {
    ///             b1
    ///         } else if y == 2 {
    ///             b2
    ///         } else {
    ///             b4
    ///         }
    ///     }
    ///     B -> b3,
    /// }
    /// ```
    ///
    /// That means that we have to collect all the branches for variant `A`,
    /// to lower them together.
    ///
    /// Default patterns have to be added to each discriminant too. Here's
    /// a particularly interesting case:
    ///
    /// ```roto
    /// match x {
    ///     A(y) | c1 -> b1,
    ///     _ | c2 -> b2,
    ///     B | c3 -> b3,
    ///     _ -> b4,
    /// }
    /// ```
    ///
    /// This should be compiled equivalently to:
    ///
    /// ```roto
    /// match x {
    ///     A(y) -> {
    ///         if c1 { b1 }
    ///         else if c2 { b2 }
    ///         else { b4 }
    ///     }
    ///     B -> {
    ///         if c2 { b2 }
    ///         else if c3 { b3 }
    ///         else { b4 }
    ///     }
    /// }
    /// ```
    ///
    /// Note how the default patterns are added to the if-else chains for
    /// all possible discriminants.
    ///
    /// We do this by checking which variants occur in patterns and then
    /// making those chains for all branches that match that discriminant or
    /// are `_`.
    pub fn r#match(&mut self, id: ast::MetaId, m: &ast::Meta<ast::Match>) -> Value {
        let ast::Match { expr, arms } = &**m;
        let Type::Name(type_name) = self.types.type_of(expr) else {
            ice!("can only match on enums")
        };
        let type_def = self.types.resolve_type_name(&type_name);
        let variants = type_def.match_patterns(&type_name.args).unwrap();

        let current_label = self.emit.current_label();
        let prefix = self.labels.label(current_label, "match");

        let default_label = self.labels.label(prefix, "default");
        let cont_label = self.labels.next(current_label);

        // First collect all the information needed to create the switches
        // to arrive at the right arm
        let branches: Vec<_> = arms
            .iter()
            .enumerate()
            .map(|(i, arm)| {
                let discriminant = match &*arm.pattern {
                    ast::Pattern::EnumVariant { variant, .. } => {
                        Some(variants.iter().position(|s| s.name == **variant).unwrap())
                    }
                    ast::Pattern::Underscore => None,
                };
                (discriminant, arm, i)
            })
            .collect();

        // Get everything we need to match on in the outer layer
        // regardless of guards.
        let all_discriminants: HashSet<_> = branches.iter().filter_map(|(d, _, _)| *d).collect();

        let all_discriminants: HashMap<_, _> = all_discriminants
            .into_iter()
            .map(|d| (d, self.labels.label(prefix, format!("case_{d}"))))
            .collect();

        let switch_branches = all_discriminants
            .iter()
            .map(|(discriminant, label)| (*discriminant, *label))
            .collect();

        // We need to know for the switch whether there are any default
        // branches. So start with this check
        let default_branches: Vec<_> = branches.iter().filter(|(d, _, _)| d.is_none()).collect();

        let examinee = self.expr_ty_val(expr);
        let examinee = self.assign_to_var(examinee);
        let discriminant = self.vars.undropped_tmp_ty(Type::discriminant());
        self.emit.assign(
            Place::from(discriminant.clone()),
            Type::discriminant(),
            Value::Discriminant(examinee.clone()),
        );
        let default_branch = if default_branches.is_empty() {
            None
        } else {
            Some(default_label)
        };
        self.emit
            .switch(discriminant, switch_branches, default_branch);

        let arm_labels: HashMap<_, _> = branches
            .iter()
            .map(|(_, _, idx)| (*idx, self.labels.label(prefix, format!("arm_{idx}"))))
            .collect();

        for (discriminant, label) in all_discriminants {
            // Each discriminant gets the branches for itself and `_`.
            // See doc comment on this function for more information.
            let branches: Vec<_> = branches
                .iter()
                .filter(|(d, _, _)| *d == Some(discriminant) || d.is_none())
                .collect();
            self.match_case(
                examinee.clone(),
                Some(&variants[discriminant]),
                label,
                &branches,
                &arm_labels,
            );
        }

        if !default_branches.is_empty() {
            self.match_case(
                examinee,
                None,
                default_label,
                &default_branches,
                &arm_labels,
            );
        }

        // Here we finally create all the blocks for the expression of each arm.
        let out = self.vars.undropped_tmp_ty(self.types.type_of(id));
        for (_, arm, arm_index) in branches {
            self.vars.push_frame();

            // Re-add the bindings generated by this pattern to the set of live
            // variables, so we drop them properly.
            if let ast::Pattern::EnumVariant {
                variant: _,
                fields: Some(fields),
            } = &*arm.pattern
            {
                for field_binding in fields.iter() {
                    let name = self.types.resolved_name(field_binding);
                    self.vars.insert(TypedVar(
                        Var::Ident(name.scope, name.ident),
                        self.types.type_of(field_binding),
                    ));
                }
            }

            self.emit.new_block(arm_labels[&arm_index]);
            let val = self.block(&arm.body);
            self.emit
                .assign(Place::from(out.clone()), out.1.clone(), val);

            let to_drop = self.vars.pop_frame();
            self.emit.drop_frame(to_drop);
            self.emit.jump(cont_label);
        }

        self.emit.new_block(cont_label);
        self.vars.add_live(out.0, out.1.clone());

        Value::Move(out)
    }

    fn match_case(
        &mut self,
        examinee: TypedVar,
        variant: Option<&EnumVariant>,
        label: LabelRef,
        branches: &[&(Option<usize>, &ast::MatchArm, usize)],
        arm_labels: &HashMap<usize, LabelRef>,
    ) {
        let examinee = Place::from(examinee);

        self.emit.new_block(label);

        let guard = self.labels.label(label, format!("guard_{}", 0));
        self.emit.jump(guard);

        let mut next_label = guard;

        for (i, &(_, arm, arm_index)) in branches.iter().enumerate() {
            let guard = next_label;
            self.emit.new_block(guard);
            self.vars.push_frame();

            // We can only extract the fields at each arm, we cannot combine
            // them unfortunately, since we cannot combine these patterns:
            //   Some(x) | Some(y) | Some(_) | _
            // with 1 extraction.
            //
            // The fields we extract get dropped in two places:
            //  - At the end of an arm that uses them.
            //  - When a guard expression evaluates to false.
            //
            // Here, that means that we drop when the guard evaluates to false
            // and otherwise just "forget".
            if let ast::Pattern::EnumVariant {
                fields: Some(fields),
                ..
            } = &*arm.pattern
            {
                let variant = variant.unwrap();
                for (index, field_binding) in fields.iter().enumerate() {
                    let name = self.types.resolved_name(field_binding);
                    let ty = self.types.type_of(field_binding);
                    let var = TypedVar(Var::Ident(name.scope, name.ident), ty.clone());
                    self.vars.add_live(var.0, var.1.clone());
                    let val = examinee.clone().with_variant(variant.name, index);
                    self.do_assign(var, TypedValue(Value::Clone(val), ty));
                }
            }

            next_label = self.labels.label(label, format!("guard_{}", i + 1));

            let arm_label = arm_labels[arm_index];

            // Even if we "forget" to drop the values, we still need to pop them from the stack.
            let to_drop = self.vars.pop_frame();

            let label_ref = if let Some(guard) = &arm.guard {
                let op = TypedValue(self.expr(guard), Type::bool());
                let op = self.assign_to_var(op);

                let intermediate_label = self.labels.label(label, format!("guard_{i}_drop"));

                self.emit.branch(op, arm_label, intermediate_label);
                self.emit.new_block(intermediate_label);
                self.emit.drop_frame(to_drop);

                next_label
            } else {
                arm_label
            };
            self.emit.jump(label_ref);
        }
    }
}

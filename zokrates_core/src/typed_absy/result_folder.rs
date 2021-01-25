// Generic walk through a typed AST. Not mutating in place

use crate::compile::CompileError;
use crate::typed_absy::types::{ArrayType, StructMember, StructType};
use crate::typed_absy::*;
use zokrates_field::Field;

pub trait ResultFolder<'ast, T: Field>: Sized {
    fn fold_program(
        &mut self,
        p: TypedProgram<'ast, T>,
    ) -> Result<TypedProgram<'ast, T>, CompileError> {
        fold_program(self, p)
    }

    fn fold_module(
        &mut self,
        p: TypedModule<'ast, T>,
    ) -> Result<TypedModule<'ast, T>, CompileError> {
        fold_module(self, p)
    }

    fn fold_function_symbol(
        &mut self,
        s: TypedFunctionSymbol<'ast, T>,
    ) -> Result<TypedFunctionSymbol<'ast, T>, CompileError> {
        fold_function_symbol(self, s)
    }

    fn fold_function(
        &mut self,
        f: TypedFunction<'ast, T>,
    ) -> Result<TypedFunction<'ast, T>, CompileError> {
        fold_function(self, f)
    }

    fn fold_parameter(
        &mut self,
        p: DeclarationParameter<'ast>,
    ) -> Result<DeclarationParameter<'ast>, CompileError> {
        Ok(DeclarationParameter {
            id: self.fold_declaration_variable(p.id)?,
            ..p
        })
    }

    fn fold_name(&mut self, n: Identifier<'ast>) -> Result<Identifier<'ast>, CompileError> {
        Ok(n)
    }

    fn fold_variable(&mut self, v: Variable<'ast, T>) -> Result<Variable<'ast, T>, CompileError> {
        Ok(Variable {
            id: self.fold_name(v.id)?,
            _type: self.fold_type(v._type)?,
        })
    }

    fn fold_declaration_variable(
        &mut self,
        v: DeclarationVariable<'ast>,
    ) -> Result<DeclarationVariable<'ast>, CompileError> {
        Ok(DeclarationVariable {
            id: self.fold_name(v.id)?,
            _type: self.fold_declaration_type(v._type)?,
        })
    }

    fn fold_type(&mut self, t: Type<'ast, T>) -> Result<Type<'ast, T>, CompileError> {
        use self::GType::*;

        match t {
            Array(array_type) => Ok(Array(self.fold_array_type(array_type)?)),
            Struct(struct_type) => Ok(Struct(self.fold_struct_type(struct_type)?)),
            t => Ok(t),
        }
    }

    fn fold_array_type(
        &mut self,
        t: ArrayType<'ast, T>,
    ) -> Result<ArrayType<'ast, T>, CompileError> {
        Ok(ArrayType {
            ty: box self.fold_type(*t.ty)?,
            size: self.fold_uint_expression(t.size)?,
        })
    }

    fn fold_struct_type(
        &mut self,
        t: StructType<'ast, T>,
    ) -> Result<StructType<'ast, T>, CompileError> {
        Ok(StructType {
            members: t
                .members
                .into_iter()
                .map(|m| {
                    let id = m.id;
                    self.fold_type(*m.ty)
                        .map(|ty| StructMember { ty: box ty, id })
                })
                .collect::<Result<_, _>>()?,
            ..t
        })
    }

    fn fold_declaration_type(
        &mut self,
        t: DeclarationType<'ast>,
    ) -> Result<DeclarationType<'ast>, CompileError> {
        Ok(t)
    }

    fn fold_assignee(
        &mut self,
        a: TypedAssignee<'ast, T>,
    ) -> Result<TypedAssignee<'ast, T>, CompileError> {
        match a {
            TypedAssignee::Identifier(v) => Ok(TypedAssignee::Identifier(self.fold_variable(v)?)),
            TypedAssignee::Select(box a, box index) => Ok(TypedAssignee::Select(
                box self.fold_assignee(a)?,
                box self.fold_uint_expression(index)?,
            )),
            TypedAssignee::Member(box s, m) => {
                Ok(TypedAssignee::Member(box self.fold_assignee(s)?, m))
            }
        }
    }

    fn fold_statement(
        &mut self,
        s: TypedStatement<'ast, T>,
    ) -> Result<Vec<TypedStatement<'ast, T>>, CompileError> {
        fold_statement(self, s)
    }

    fn fold_expression_or_spread(
        &mut self,
        e: TypedExpressionOrSpread<'ast, T>,
    ) -> Result<TypedExpressionOrSpread<'ast, T>, CompileError> {
        Ok(match e {
            TypedExpressionOrSpread::Expression(e) => {
                TypedExpressionOrSpread::Expression(self.fold_expression(e)?)
            }
            TypedExpressionOrSpread::Spread(s) => {
                TypedExpressionOrSpread::Spread(self.fold_spread(s)?)
            }
        })
    }

    fn fold_spread(
        &mut self,
        s: TypedSpread<'ast, T>,
    ) -> Result<TypedSpread<'ast, T>, CompileError> {
        Ok(TypedSpread {
            array: self.fold_array_expression(s.array)?,
        })
    }

    fn fold_expression(
        &mut self,
        e: TypedExpression<'ast, T>,
    ) -> Result<TypedExpression<'ast, T>, CompileError> {
        match e {
            TypedExpression::FieldElement(e) => Ok(self.fold_field_expression(e)?.into()),
            TypedExpression::Boolean(e) => Ok(self.fold_boolean_expression(e)?.into()),
            TypedExpression::Uint(e) => Ok(self.fold_uint_expression(e)?.into()),
            TypedExpression::Array(e) => Ok(self.fold_array_expression(e)?.into()),
            TypedExpression::Struct(e) => Ok(self.fold_struct_expression(e)?.into()),
            TypedExpression::Int(e) => Ok(self.fold_int_expression(e)?.into()),
        }
    }

    fn fold_array_expression(
        &mut self,
        e: ArrayExpression<'ast, T>,
    ) -> Result<ArrayExpression<'ast, T>, CompileError> {
        fold_array_expression(self, e)
    }

    fn fold_struct_expression(
        &mut self,
        e: StructExpression<'ast, T>,
    ) -> Result<StructExpression<'ast, T>, CompileError> {
        fold_struct_expression(self, e)
    }

    fn fold_expression_list(
        &mut self,
        es: TypedExpressionList<'ast, T>,
    ) -> Result<TypedExpressionList<'ast, T>, CompileError> {
        fold_expression_list(self, es)
    }

    fn fold_int_expression(
        &mut self,
        e: IntExpression<'ast, T>,
    ) -> Result<IntExpression<'ast, T>, CompileError> {
        fold_int_expression(self, e)
    }

    fn fold_field_expression(
        &mut self,
        e: FieldElementExpression<'ast, T>,
    ) -> Result<FieldElementExpression<'ast, T>, CompileError> {
        fold_field_expression(self, e)
    }
    fn fold_boolean_expression(
        &mut self,
        e: BooleanExpression<'ast, T>,
    ) -> Result<BooleanExpression<'ast, T>, CompileError> {
        fold_boolean_expression(self, e)
    }
    fn fold_uint_expression(
        &mut self,
        e: UExpression<'ast, T>,
    ) -> Result<UExpression<'ast, T>, CompileError> {
        fold_uint_expression(self, e)
    }

    fn fold_uint_expression_inner(
        &mut self,
        bitwidth: UBitwidth,
        e: UExpressionInner<'ast, T>,
    ) -> Result<UExpressionInner<'ast, T>, CompileError> {
        fold_uint_expression_inner(self, bitwidth, e)
    }

    fn fold_array_expression_inner(
        &mut self,
        ty: &ArrayType<'ast, T>,
        e: ArrayExpressionInner<'ast, T>,
    ) -> Result<ArrayExpressionInner<'ast, T>, CompileError> {
        fold_array_expression_inner(self, ty, e)
    }
    fn fold_struct_expression_inner(
        &mut self,
        ty: &StructType<'ast, T>,
        e: StructExpressionInner<'ast, T>,
    ) -> Result<StructExpressionInner<'ast, T>, CompileError> {
        fold_struct_expression_inner(self, ty, e)
    }
}

pub fn fold_statement<'ast, T: Field, F: ResultFolder<'ast, T>>(
    f: &mut F,
    s: TypedStatement<'ast, T>,
) -> Result<Vec<TypedStatement<'ast, T>>, CompileError> {
    let res = match s {
        TypedStatement::Return(expressions) => TypedStatement::Return(
            expressions
                .into_iter()
                .map(|e| f.fold_expression(e))
                .collect::<Result<_, _>>()?,
        ),
        TypedStatement::Definition(a, e) => {
            TypedStatement::Definition(f.fold_assignee(a)?, f.fold_expression(e)?)
        }
        TypedStatement::Declaration(v) => TypedStatement::Declaration(f.fold_variable(v)?),
        TypedStatement::Assertion(e) => TypedStatement::Assertion(f.fold_boolean_expression(e)?),
        TypedStatement::For(v, from, to, statements) => TypedStatement::For(
            f.fold_variable(v)?,
            f.fold_uint_expression(from)?,
            f.fold_uint_expression(to)?,
            statements
                .into_iter()
                .map(|s| f.fold_statement(s))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .flatten()
                .collect(),
        ),
        TypedStatement::MultipleDefinition(variables, elist) => TypedStatement::MultipleDefinition(
            variables
                .into_iter()
                .map(|v| f.fold_assignee(v))
                .collect::<Result<_, _>>()?,
            f.fold_expression_list(elist)?,
        ),
        s => s,
    };
    Ok(vec![res])
}

pub fn fold_array_expression_inner<'ast, T: Field, F: ResultFolder<'ast, T>>(
    f: &mut F,
    _: &ArrayType<'ast, T>,
    e: ArrayExpressionInner<'ast, T>,
) -> Result<ArrayExpressionInner<'ast, T>, CompileError> {
    let e = match e {
        ArrayExpressionInner::Identifier(id) => ArrayExpressionInner::Identifier(f.fold_name(id)?),
        ArrayExpressionInner::Value(exprs) => ArrayExpressionInner::Value(
            exprs
                .into_iter()
                .map(|e| f.fold_expression_or_spread(e))
                .collect::<Result<_, _>>()?,
        ),
        ArrayExpressionInner::FunctionCall(id, generics, exps) => {
            let generics = generics
                .into_iter()
                .map(|g| g.map(|g| f.fold_uint_expression(g)).transpose())
                .collect::<Result<_, _>>()?;
            let exps = exps
                .into_iter()
                .map(|e| f.fold_expression(e))
                .collect::<Result<_, _>>()?;
            ArrayExpressionInner::FunctionCall(id, generics, exps)
        }
        ArrayExpressionInner::IfElse(box condition, box consequence, box alternative) => {
            ArrayExpressionInner::IfElse(
                box f.fold_boolean_expression(condition)?,
                box f.fold_array_expression(consequence)?,
                box f.fold_array_expression(alternative)?,
            )
        }
        ArrayExpressionInner::Member(box s, id) => {
            let s = f.fold_struct_expression(s)?;
            ArrayExpressionInner::Member(box s, id)
        }
        ArrayExpressionInner::Select(box array, box index) => {
            let array = f.fold_array_expression(array)?;
            let index = f.fold_uint_expression(index)?;
            ArrayExpressionInner::Select(box array, box index)
        }
        ArrayExpressionInner::Slice(box array, box from, box to) => {
            let array = f.fold_array_expression(array)?;
            let from = f.fold_uint_expression(from)?;
            let to = f.fold_uint_expression(to)?;
            ArrayExpressionInner::Slice(box array, box from, box to)
        }
        ArrayExpressionInner::Repeat(box e, box count) => {
            let e = f.fold_expression(e)?;
            let count = f.fold_uint_expression(count)?;
            ArrayExpressionInner::Repeat(box e, box count)
        }
    };
    Ok(e)
}

pub fn fold_struct_expression_inner<'ast, T: Field, F: ResultFolder<'ast, T>>(
    f: &mut F,
    _: &StructType<'ast, T>,
    e: StructExpressionInner<'ast, T>,
) -> Result<StructExpressionInner<'ast, T>, CompileError> {
    let e = match e {
        StructExpressionInner::Identifier(id) => {
            StructExpressionInner::Identifier(f.fold_name(id)?)
        }
        StructExpressionInner::Value(exprs) => StructExpressionInner::Value(
            exprs
                .into_iter()
                .map(|e| f.fold_expression(e))
                .collect::<Result<_, _>>()?,
        ),
        StructExpressionInner::FunctionCall(id, generics, exps) => {
            let generics = generics
                .into_iter()
                .map(|g| g.map(|g| f.fold_uint_expression(g)).transpose())
                .collect::<Result<_, _>>()?;
            let exps = exps
                .into_iter()
                .map(|e| f.fold_expression(e))
                .collect::<Result<_, _>>()?;
            StructExpressionInner::FunctionCall(id, generics, exps)
        }
        StructExpressionInner::IfElse(box condition, box consequence, box alternative) => {
            StructExpressionInner::IfElse(
                box f.fold_boolean_expression(condition)?,
                box f.fold_struct_expression(consequence)?,
                box f.fold_struct_expression(alternative)?,
            )
        }
        StructExpressionInner::Member(box s, id) => {
            let s = f.fold_struct_expression(s)?;
            StructExpressionInner::Member(box s, id)
        }
        StructExpressionInner::Select(box array, box index) => {
            let array = f.fold_array_expression(array)?;
            let index = f.fold_uint_expression(index)?;
            StructExpressionInner::Select(box array, box index)
        }
    };
    Ok(e)
}

pub fn fold_field_expression<'ast, T: Field, F: ResultFolder<'ast, T>>(
    f: &mut F,
    e: FieldElementExpression<'ast, T>,
) -> Result<FieldElementExpression<'ast, T>, CompileError> {
    let e = match e {
        FieldElementExpression::Number(n) => FieldElementExpression::Number(n),
        FieldElementExpression::Identifier(id) => {
            FieldElementExpression::Identifier(f.fold_name(id)?)
        }
        FieldElementExpression::Add(box e1, box e2) => {
            let e1 = f.fold_field_expression(e1)?;
            let e2 = f.fold_field_expression(e2)?;
            FieldElementExpression::Add(box e1, box e2)
        }
        FieldElementExpression::Sub(box e1, box e2) => {
            let e1 = f.fold_field_expression(e1)?;
            let e2 = f.fold_field_expression(e2)?;
            FieldElementExpression::Sub(box e1, box e2)
        }
        FieldElementExpression::Mult(box e1, box e2) => {
            let e1 = f.fold_field_expression(e1)?;
            let e2 = f.fold_field_expression(e2)?;
            FieldElementExpression::Mult(box e1, box e2)
        }
        FieldElementExpression::Div(box e1, box e2) => {
            let e1 = f.fold_field_expression(e1)?;
            let e2 = f.fold_field_expression(e2)?;
            FieldElementExpression::Div(box e1, box e2)
        }
        FieldElementExpression::Pow(box e1, box e2) => {
            let e1 = f.fold_field_expression(e1)?;
            let e2 = f.fold_uint_expression(e2)?;
            FieldElementExpression::Pow(box e1, box e2)
        }
        FieldElementExpression::IfElse(box cond, box cons, box alt) => {
            let cond = f.fold_boolean_expression(cond)?;
            let cons = f.fold_field_expression(cons)?;
            let alt = f.fold_field_expression(alt)?;
            FieldElementExpression::IfElse(box cond, box cons, box alt)
        }
        FieldElementExpression::FunctionCall(key, generics, exps) => {
            let generics = generics
                .into_iter()
                .map(|g| g.map(|g| f.fold_uint_expression(g)).transpose())
                .collect::<Result<_, _>>()?;
            let exps = exps
                .into_iter()
                .map(|e| f.fold_expression(e))
                .collect::<Result<_, _>>()?;
            FieldElementExpression::FunctionCall(key, generics, exps)
        }
        FieldElementExpression::Member(box s, id) => {
            let s = f.fold_struct_expression(s)?;
            FieldElementExpression::Member(box s, id)
        }
        FieldElementExpression::Select(box array, box index) => {
            let array = f.fold_array_expression(array)?;
            let index = f.fold_uint_expression(index)?;
            FieldElementExpression::Select(box array, box index)
        }
    };
    Ok(e)
}

pub fn fold_int_expression<'ast, T: Field, F: ResultFolder<'ast, T>>(
    _: &mut F,
    _: IntExpression<'ast, T>,
) -> Result<IntExpression<'ast, T>, CompileError> {
    unreachable!()
}

pub fn fold_boolean_expression<'ast, T: Field, F: ResultFolder<'ast, T>>(
    f: &mut F,
    e: BooleanExpression<'ast, T>,
) -> Result<BooleanExpression<'ast, T>, CompileError> {
    let e = match e {
        BooleanExpression::Value(v) => BooleanExpression::Value(v),
        BooleanExpression::Identifier(id) => BooleanExpression::Identifier(f.fold_name(id)?),
        BooleanExpression::FieldEq(box e1, box e2) => {
            let e1 = f.fold_field_expression(e1)?;
            let e2 = f.fold_field_expression(e2)?;
            BooleanExpression::FieldEq(box e1, box e2)
        }
        BooleanExpression::BoolEq(box e1, box e2) => {
            let e1 = f.fold_boolean_expression(e1)?;
            let e2 = f.fold_boolean_expression(e2)?;
            BooleanExpression::BoolEq(box e1, box e2)
        }
        BooleanExpression::ArrayEq(box e1, box e2) => {
            let e1 = f.fold_array_expression(e1)?;
            let e2 = f.fold_array_expression(e2)?;
            BooleanExpression::ArrayEq(box e1, box e2)
        }
        BooleanExpression::StructEq(box e1, box e2) => {
            let e1 = f.fold_struct_expression(e1)?;
            let e2 = f.fold_struct_expression(e2)?;
            BooleanExpression::StructEq(box e1, box e2)
        }
        BooleanExpression::UintEq(box e1, box e2) => {
            let e1 = f.fold_uint_expression(e1)?;
            let e2 = f.fold_uint_expression(e2)?;
            BooleanExpression::UintEq(box e1, box e2)
        }
        BooleanExpression::FieldLt(box e1, box e2) => {
            let e1 = f.fold_field_expression(e1)?;
            let e2 = f.fold_field_expression(e2)?;
            BooleanExpression::FieldLt(box e1, box e2)
        }
        BooleanExpression::FieldLe(box e1, box e2) => {
            let e1 = f.fold_field_expression(e1)?;
            let e2 = f.fold_field_expression(e2)?;
            BooleanExpression::FieldLe(box e1, box e2)
        }
        BooleanExpression::FieldGt(box e1, box e2) => {
            let e1 = f.fold_field_expression(e1)?;
            let e2 = f.fold_field_expression(e2)?;
            BooleanExpression::FieldGt(box e1, box e2)
        }
        BooleanExpression::FieldGe(box e1, box e2) => {
            let e1 = f.fold_field_expression(e1)?;
            let e2 = f.fold_field_expression(e2)?;
            BooleanExpression::FieldGe(box e1, box e2)
        }
        BooleanExpression::UintLt(box e1, box e2) => {
            let e1 = f.fold_uint_expression(e1)?;
            let e2 = f.fold_uint_expression(e2)?;
            BooleanExpression::UintLt(box e1, box e2)
        }
        BooleanExpression::UintLe(box e1, box e2) => {
            let e1 = f.fold_uint_expression(e1)?;
            let e2 = f.fold_uint_expression(e2)?;
            BooleanExpression::UintLe(box e1, box e2)
        }
        BooleanExpression::UintGt(box e1, box e2) => {
            let e1 = f.fold_uint_expression(e1)?;
            let e2 = f.fold_uint_expression(e2)?;
            BooleanExpression::UintGt(box e1, box e2)
        }
        BooleanExpression::UintGe(box e1, box e2) => {
            let e1 = f.fold_uint_expression(e1)?;
            let e2 = f.fold_uint_expression(e2)?;
            BooleanExpression::UintGe(box e1, box e2)
        }
        BooleanExpression::Or(box e1, box e2) => {
            let e1 = f.fold_boolean_expression(e1)?;
            let e2 = f.fold_boolean_expression(e2)?;
            BooleanExpression::Or(box e1, box e2)
        }
        BooleanExpression::And(box e1, box e2) => {
            let e1 = f.fold_boolean_expression(e1)?;
            let e2 = f.fold_boolean_expression(e2)?;
            BooleanExpression::And(box e1, box e2)
        }
        BooleanExpression::Not(box e) => {
            let e = f.fold_boolean_expression(e)?;
            BooleanExpression::Not(box e)
        }
        BooleanExpression::FunctionCall(key, generics, exps) => {
            let generics = generics
                .into_iter()
                .map(|g| g.map(|g| f.fold_uint_expression(g)).transpose())
                .collect::<Result<_, _>>()?;
            let exps = exps
                .into_iter()
                .map(|e| f.fold_expression(e))
                .collect::<Result<_, _>>()?;
            BooleanExpression::FunctionCall(key, generics, exps)
        }
        BooleanExpression::IfElse(box cond, box cons, box alt) => {
            let cond = f.fold_boolean_expression(cond)?;
            let cons = f.fold_boolean_expression(cons)?;
            let alt = f.fold_boolean_expression(alt)?;
            BooleanExpression::IfElse(box cond, box cons, box alt)
        }
        BooleanExpression::Member(box s, id) => {
            let s = f.fold_struct_expression(s)?;
            BooleanExpression::Member(box s, id)
        }
        BooleanExpression::Select(box array, box index) => {
            let array = f.fold_array_expression(array)?;
            let index = f.fold_uint_expression(index)?;
            BooleanExpression::Select(box array, box index)
        }
    };
    Ok(e)
}

pub fn fold_uint_expression<'ast, T: Field, F: ResultFolder<'ast, T>>(
    f: &mut F,
    e: UExpression<'ast, T>,
) -> Result<UExpression<'ast, T>, CompileError> {
    Ok(UExpression {
        inner: f.fold_uint_expression_inner(e.bitwidth, e.inner)?,
        ..e
    })
}

pub fn fold_uint_expression_inner<'ast, T: Field, F: ResultFolder<'ast, T>>(
    f: &mut F,
    _: UBitwidth,
    e: UExpressionInner<'ast, T>,
) -> Result<UExpressionInner<'ast, T>, CompileError> {
    let e = match e {
        UExpressionInner::Value(v) => UExpressionInner::Value(v),
        UExpressionInner::Identifier(id) => UExpressionInner::Identifier(f.fold_name(id)?),
        UExpressionInner::Add(box left, box right) => {
            let left = f.fold_uint_expression(left)?;
            let right = f.fold_uint_expression(right)?;

            UExpressionInner::Add(box left, box right)
        }
        UExpressionInner::Sub(box left, box right) => {
            let left = f.fold_uint_expression(left)?;
            let right = f.fold_uint_expression(right)?;

            UExpressionInner::Sub(box left, box right)
        }
        UExpressionInner::FloorSub(box left, box right) => {
            let left = f.fold_uint_expression(left)?;
            let right = f.fold_uint_expression(right)?;

            UExpressionInner::FloorSub(box left, box right)
        }
        UExpressionInner::Mult(box left, box right) => {
            let left = f.fold_uint_expression(left)?;
            let right = f.fold_uint_expression(right)?;

            UExpressionInner::Mult(box left, box right)
        }
        UExpressionInner::Div(box left, box right) => {
            let left = f.fold_uint_expression(left)?;
            let right = f.fold_uint_expression(right)?;

            UExpressionInner::Div(box left, box right)
        }
        UExpressionInner::Rem(box left, box right) => {
            let left = f.fold_uint_expression(left)?;
            let right = f.fold_uint_expression(right)?;

            UExpressionInner::Rem(box left, box right)
        }
        UExpressionInner::Xor(box left, box right) => {
            let left = f.fold_uint_expression(left)?;
            let right = f.fold_uint_expression(right)?;

            UExpressionInner::Xor(box left, box right)
        }
        UExpressionInner::And(box left, box right) => {
            let left = f.fold_uint_expression(left)?;
            let right = f.fold_uint_expression(right)?;

            UExpressionInner::And(box left, box right)
        }
        UExpressionInner::Or(box left, box right) => {
            let left = f.fold_uint_expression(left)?;
            let right = f.fold_uint_expression(right)?;

            UExpressionInner::Or(box left, box right)
        }
        UExpressionInner::LeftShift(box e, box by) => {
            let e = f.fold_uint_expression(e)?;
            let by = f.fold_field_expression(by)?;

            UExpressionInner::LeftShift(box e, box by)
        }
        UExpressionInner::RightShift(box e, box by) => {
            let e = f.fold_uint_expression(e)?;
            let by = f.fold_field_expression(by)?;

            UExpressionInner::RightShift(box e, box by)
        }
        UExpressionInner::Not(box e) => {
            let e = f.fold_uint_expression(e)?;

            UExpressionInner::Not(box e)
        }
        UExpressionInner::FunctionCall(key, generics, exps) => {
            let generics = generics
                .into_iter()
                .map(|g| g.map(|g| f.fold_uint_expression(g)).transpose())
                .collect::<Result<_, _>>()?;
            let exps = exps
                .into_iter()
                .map(|e| f.fold_expression(e))
                .collect::<Result<_, _>>()?;
            UExpressionInner::FunctionCall(key, generics, exps)
        }
        UExpressionInner::Select(box array, box index) => {
            let array = f.fold_array_expression(array)?;
            let index = f.fold_uint_expression(index)?;
            UExpressionInner::Select(box array, box index)
        }
        UExpressionInner::IfElse(box cond, box cons, box alt) => {
            let cond = f.fold_boolean_expression(cond)?;
            let cons = f.fold_uint_expression(cons)?;
            let alt = f.fold_uint_expression(alt)?;
            UExpressionInner::IfElse(box cond, box cons, box alt)
        }
        UExpressionInner::Member(box s, id) => {
            let s = f.fold_struct_expression(s)?;
            UExpressionInner::Member(box s, id)
        }
    };
    Ok(e)
}

pub fn fold_function<'ast, T: Field, F: ResultFolder<'ast, T>>(
    f: &mut F,
    fun: TypedFunction<'ast, T>,
) -> Result<TypedFunction<'ast, T>, CompileError> {
    Ok(TypedFunction {
        arguments: fun
            .arguments
            .into_iter()
            .map(|a| f.fold_parameter(a))
            .collect::<Result<_, _>>()?,
        statements: fun
            .statements
            .into_iter()
            .map(|s| f.fold_statement(s))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect(),
        ..fun
    })
}

pub fn fold_array_expression<'ast, T: Field, F: ResultFolder<'ast, T>>(
    f: &mut F,
    e: ArrayExpression<'ast, T>,
) -> Result<ArrayExpression<'ast, T>, CompileError> {
    let ty = f.fold_array_type(*e.ty)?;

    Ok(ArrayExpression {
        inner: f.fold_array_expression_inner(&ty, e.inner)?,
        ty: box ty,
    })
}

pub fn fold_expression_list<'ast, T: Field, F: ResultFolder<'ast, T>>(
    f: &mut F,
    es: TypedExpressionList<'ast, T>,
) -> Result<TypedExpressionList<'ast, T>, CompileError> {
    match es {
        TypedExpressionList::FunctionCall(id, generics, arguments, types) => {
            let generics = generics
                .into_iter()
                .map(|g| g.map(|g| f.fold_uint_expression(g)).transpose())
                .collect::<Result<_, _>>()?;
            Ok(TypedExpressionList::FunctionCall(
                id,
                generics,
                arguments
                    .into_iter()
                    .map(|a| f.fold_expression(a))
                    .collect::<Result<_, _>>()?,
                types
                    .into_iter()
                    .map(|t| f.fold_type(t))
                    .collect::<Result<_, _>>()?,
            ))
        }
    }
}

pub fn fold_struct_expression<'ast, T: Field, F: ResultFolder<'ast, T>>(
    f: &mut F,
    e: StructExpression<'ast, T>,
) -> Result<StructExpression<'ast, T>, CompileError> {
    Ok(StructExpression {
        inner: f.fold_struct_expression_inner(&e.ty, e.inner)?,
        ..e
    })
}

pub fn fold_function_symbol<'ast, T: Field, F: ResultFolder<'ast, T>>(
    f: &mut F,
    s: TypedFunctionSymbol<'ast, T>,
) -> Result<TypedFunctionSymbol<'ast, T>, CompileError> {
    match s {
        TypedFunctionSymbol::Here(fun) => Ok(TypedFunctionSymbol::Here(f.fold_function(fun)?)),
        there => Ok(there), // by default, do not fold modules recursively
    }
}

pub fn fold_module<'ast, T: Field, F: ResultFolder<'ast, T>>(
    f: &mut F,
    p: TypedModule<'ast, T>,
) -> Result<TypedModule<'ast, T>, CompileError> {
    Ok(TypedModule {
        functions: p
            .functions
            .into_iter()
            .map(|(key, fun)| f.fold_function_symbol(fun).map(|f| (key, f)))
            .collect::<Result<_, _>>()?,
    })
}

pub fn fold_program<'ast, T: Field, F: ResultFolder<'ast, T>>(
    f: &mut F,
    p: TypedProgram<'ast, T>,
) -> Result<TypedProgram<'ast, T>, CompileError> {
    Ok(TypedProgram {
        modules: p
            .modules
            .into_iter()
            .map(|(module_id, module)| f.fold_module(module).map(|m| (module_id, m)))
            .collect::<Result<_, _>>()?,
        main: p.main,
    })
}
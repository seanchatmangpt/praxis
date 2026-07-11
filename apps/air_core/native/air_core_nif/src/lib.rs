use rustler::{Encoder, Env, NifResult, Term};

rustler::atoms! {
    undefined,
    ok,
}

#[rustler::nif]
fn eval_expr_nif<'a>(
    env: Env<'a>,
    expr: Term<'a>,
    map_env: Term<'a>,
    result_term: Term<'a>,
) -> NifResult<Term<'a>> {
    eval_expr_internal(env, expr, map_env, result_term)
}

fn eval_expr_internal<'a>(
    env: Env<'a>,
    expr: Term<'a>,
    map_env: Term<'a>,
    result_term: Term<'a>,
) -> NifResult<Term<'a>> {
    let tuple: Vec<Term<'a>> = rustler::types::tuple::get_tuple(expr)?;
    if tuple.is_empty() {
        return Err(rustler::Error::BadArg);
    }

    let tag = tuple[0].atom_to_string()?;
    match tag.as_str() {
        "literal" => Ok(tuple[1]),
        "var" => {
            let name_term = tuple[1];
            let mut is_result = false;

            if let Ok(name_str) = name_term.atom_to_string() {
                if name_str == "__result__" {
                    is_result = true;
                }
            } else if let Ok(name_bin) = name_term.decode::<rustler::Binary>() {
                if name_bin.as_slice() == b"__result__" {
                    is_result = true;
                }
            }

            if is_result {
                if result_term.get_type() != rustler::TermType::Atom {
                    return Ok(result_term);
                }
                if let Ok(res_str) = result_term.atom_to_string() {
                    if res_str != "undefined" {
                        return Ok(result_term);
                    }
                }
            }

            if let Ok(val) = map_env.map_get(name_term) {
                Ok(val)
            } else {
                Ok(undefined().to_term(env))
            }
        }
        "op" => {
            let op = tuple[1].atom_to_string()?;
            if tuple.len() == 4 {
                let v1 = eval_expr_internal(env, tuple[2], map_env, result_term)?;
                let v2 = eval_expr_internal(env, tuple[3], map_env, result_term)?;

                match op.as_str() {
                    "==" => Ok((v1.cmp(&v2) == std::cmp::Ordering::Equal).encode(env)),
                    "!=" => Ok((v1.cmp(&v2) != std::cmp::Ordering::Equal).encode(env)),
                    ">" => Ok((v1.cmp(&v2) == std::cmp::Ordering::Greater).encode(env)),
                    "<" => Ok((v1.cmp(&v2) == std::cmp::Ordering::Less).encode(env)),
                    ">=" => Ok((v1.cmp(&v2) != std::cmp::Ordering::Less).encode(env)),
                    "<=" => Ok((v1.cmp(&v2) != std::cmp::Ordering::Greater).encode(env)),
                    "and" => Ok((v1.decode::<bool>()? && v2.decode::<bool>()?).encode(env)),
                    "or" => Ok((v1.decode::<bool>()? || v2.decode::<bool>()?).encode(env)),
                    "+" => {
                        let i1: i64 = v1.decode()?;
                        let i2: i64 = v2.decode()?;
                        Ok((i1 + i2).encode(env))
                    }
                    "-" => {
                        let i1: i64 = v1.decode()?;
                        let i2: i64 = v2.decode()?;
                        Ok((i1 - i2).encode(env))
                    }
                    "*" => {
                        let i1: i64 = v1.decode()?;
                        let i2: i64 = v2.decode()?;
                        Ok((i1 * i2).encode(env))
                    }
                    "/" => {
                        let i1: i64 = v1.decode()?;
                        let i2: i64 = v2.decode()?;
                        Ok((i1 / i2).encode(env))
                    }
                    _ => Err(rustler::Error::BadArg),
                }
            } else if tuple.len() == 3 {
                let v = eval_expr_internal(env, tuple[2], map_env, result_term)?;
                match op.as_str() {
                    "not" => Ok((!v.decode::<bool>()?).encode(env)),
                    _ => Err(rustler::Error::BadArg),
                }
            } else {
                Err(rustler::Error::BadArg)
            }
        }
        _ => Err(rustler::Error::BadArg),
    }
}

rustler::init!("air_core", [eval_expr_nif]);

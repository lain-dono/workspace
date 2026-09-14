use super::{Context, ScriptResult};
use bevy::reflect::{Access, OffsetAccess, ParsedPath};
use bevy::{ptr::PtrMut, reflect::GetPath};
use std::fmt::Write;

pub fn op_debug(mut ctx: Context, imm: PtrMut) -> ScriptResult {
    type StatRef<'a> = (&'a crate::Stat, &'a crate::StatAsset);

    let format = unsafe {
        let len = *imm.as_ptr() as usize;
        let data = imm.byte_add(1).as_ptr();
        std::str::from_utf8_unchecked(std::slice::from_raw_parts(data, len))
    };

    let mut output = String::new();
    for item in PrintSegment::parse(format) {
        match item {
            PrintSegment::Lit(lit) => output.push_str(lit),
            PrintSegment::Key(key) => {
                let mut path = ParsedPath::parse(key).unwrap();
                path.0.reverse();

                let Some(OffsetAccess { access, .. }) = path.0.pop() else {
                    continue;
                };

                match access {
                    Access::Field(var) => {
                        let (stat_component, stat_asset) =
                            unsafe { ctx.local_ref::<StatRef>(&var).unwrap() };

                        path.0.reverse();

                        let value = stat_component.reflect_path(&path);
                        let value = value.or_else(|_| stat_asset.reflect_path(&path));
                        let Ok(value) = value else {
                            continue;
                        };
                        if let Some(string) = value.try_downcast_ref::<String>() {
                            output.push_str(string);
                        } else {
                            output.write_fmt(format_args!("{value:?}")).unwrap();
                        }
                    }
                    Access::ListIndex(index) => {
                        let Ok(value) = ctx.peek_prev(index) else {
                            continue;
                        };
                        if let Some(string) = value.try_downcast_ref::<String>() {
                            output.push_str(string);
                        } else {
                            output.write_fmt(format_args!("{value:?}")).unwrap();
                        }
                    }
                    _ => continue,
                }
            }
        }
    }

    println!("{output}");

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrintSegment<'a> {
    Lit(&'a str),
    Key(&'a str),
}

impl<'a> PrintSegment<'a> {
    pub fn parse(mut input: &'a str) -> impl Iterator<Item = Self> {
        let mut is_key = false;

        std::iter::from_fn(move || {
            if input.is_empty() {
                None
            } else if is_key {
                return match input.strip_prefix('{') {
                    Some(rest) => Some(Self::Lit(match rest.split_once('{') {
                        Some((prefix, rest)) => {
                            let lit = &input[..prefix.len() + 1];
                            input = rest;
                            lit
                        }
                        None => {
                            is_key = false;
                            core::mem::take(&mut input)
                        }
                    })),
                    None => input.split_once('}').map(|(key, rest)| {
                        is_key = false;
                        input = rest;
                        Self::Key(key)
                    }),
                };
            } else {
                Some(Self::Lit(match input.split_once('{') {
                    Some((prefix, rest)) => {
                        is_key = true;
                        input = rest;
                        prefix
                    }
                    None => core::mem::take(&mut input),
                }))
            }
        })
    }
}

#[test]
fn parse() {
    use PrintSegment::{Key, Lit};

    let input = "prefix {some key} suffix";
    let data = PrintSegment::parse(input).collect::<Vec<_>>();
    assert_eq!(data, [Lit("prefix "), Key("some key"), Lit(" suffix")]);

    let input = "prefix {some key suffix";
    let data = PrintSegment::parse(input).collect::<Vec<_>>();
    assert_eq!(data, [Lit("prefix ")]);
}

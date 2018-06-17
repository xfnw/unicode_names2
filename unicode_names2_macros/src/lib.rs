//! A macro that maps unicode names to chars and strings.

#![crate_type="dylib"]

#![feature(quote, plugin_registrar, plugin, rustc_private)]

extern crate syntax;
extern crate syntax_pos;
extern crate rustc;
extern crate rustc_plugin;

extern crate regex;

extern crate unicode_names2;

use syntax::ast;
use syntax::tokenstream::TokenTree;
use syntax::codemap;
use syntax_pos::symbol::Symbol;
use syntax::ext::base::{self, ExtCtxt, MacResult, MacEager, DummyResult};
use syntax::ext::build::AstBuilder;
use rustc_plugin::Registry;

#[plugin_registrar]
#[doc(hidden)]
pub fn plugin_registrar(registrar: &mut Registry) {
    registrar.register_macro("named_char", named_char);
    registrar.register_macro("named", named);
}
fn named_char(cx: &mut ExtCtxt, sp: codemap::Span,
              tts: &[TokenTree]) -> Box<MacResult+'static> {
    match base::get_single_str_from_tts(cx, sp, tts, "named_char") {
        None => {}
        Some(name) => match unicode_names2::character(&name) {
            None => cx.span_err(sp, &format!("`{}` does not name a character", name)),

            // everything worked!
            Some(c) => return MacEager::expr(cx.expr_lit(sp, ast::LitKind::Char(c))),
        }
    }
    // failed :(
    DummyResult::expr(sp)
}
fn named(cx: &mut ExtCtxt, sp: codemap::Span, tts: &[TokenTree]) -> Box<MacResult+'static> {
    let string = match base::get_single_str_from_tts(cx, sp, tts, "named") {
        None => return DummyResult::expr(sp),
        Some(s) => s
    };

     // make sure unclosed braces don't escape.
    let names_re = regex::Regex::new(r"\\N\{(.*?)(?:\}|$)").unwrap();

    let new = names_re.replace_all(&string, |c: &regex::Captures| {
        let full = c.at(0).unwrap();
        if !full.ends_with("}") {
            cx.span_err(sp, &format!("unclosed escape in `named!`: {}", full));
        } else {
            let name = c.at(1).unwrap();
            match unicode_names2::character(name) {
                Some(c) => return c.to_string(),
                None => {
                    cx.span_err(sp, &format!("`{}` does not name a character", name));
                }
            }
        }
        // failed :(
        String::new()
    });

    MacEager::expr(cx.expr_str(sp, Symbol::intern(&new)))
}
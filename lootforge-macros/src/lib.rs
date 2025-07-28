use std::iter;

use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};
use TokenTree::*;

#[proc_macro]
pub fn tooltip(input: TokenStream) -> TokenStream {
    let input_string = unwrap_string(input);
    let tokens = tokenize_string(input_string);
    wrap_tokens(tokens)
}

fn unwrap_string(input: TokenStream) -> String {
    let token_trees: Vec<_> = input.clone().into_iter().collect();
    if token_trees.len() != 1 {
        panic!("tooltip macro expect exactly one string argument");
    }
    match &token_trees[0] {
        Literal(literal) => literal.to_string(),
        _ => panic!("tooltip macro expect exactly one string argument"),
    }
}

#[derive(Debug)]
enum Token {
    Text(String),
    Roll,
    Range,
    Bleed,
    Fracture,
    Madness,
    Void,
}
impl Token {
    fn from(s: &str) -> Self {
        match s {
            ""         => Token::Text("%".to_owned()),
            "roll"     => Token::Roll,
            "range"    => Token::Range,
            "bleed"    => Token::Bleed,
            "fracture" => Token::Fracture,
            "madness"  => Token::Madness,
            "void"     => Token::Void,
            _ => panic!("unknown token: {}", s)
        }
    }
}

fn tokenize_string(input: String) -> Vec<Token> {
    let mut tokens = Vec::new();

    let mut s: &str = &input[1..input.len() - 1];
    while let Some((text, rest)) = s.split_once('%') {
        tokens.push(Token::Text(text.to_owned()));

        let token_end = rest.find(|c: char| !c.is_ascii_alphabetic()).unwrap_or(rest.len());
        let (token, rest) = rest.split_at(token_end);
        tokens.push(Token::from(token));
        s = rest;
    }
    if s.len() > 0 {
        tokens.push(Token::Text(s.to_owned()));
    }
    tokens
}

fn wrap_tokens(tokens: Vec<Token>) -> TokenStream {
    [
        Ident(Ident::new("ui", Span::call_site())),
        Punct(Punct::new('.', Spacing::Alone)),
        Ident(Ident::new("horizontal_wrapped", Span::call_site())),
        Group(Group::new(Delimiter::Parenthesis, [
            Punct(Punct::new('|', Spacing::Alone)),
            Ident(Ident::new("ui", Span::call_site())),
            Punct(Punct::new('|', Spacing::Alone)),
            Group(Group::new(Delimiter::Brace, 
                call("ui", "spacing_mut", TokenStream::new())
                .chain([                    
                    Punct(Punct::new('.', Spacing::Alone)),
                    Ident(Ident::new("item_spacing", Span::call_site())),
                    Punct(Punct::new('=', Spacing::Alone)),
                    Ident(Ident::new("vec2", Span::call_site())),
                    Group(Group::new(Delimiter::Parenthesis, [
                        Literal(Literal::f32_unsuffixed(0.)),
                        Punct(Punct::new(',', Spacing::Alone)),
                        Literal(Literal::f32_unsuffixed(0.)),
                    ].into_iter().collect())),
                    Punct(Punct::new(';', Spacing::Alone)),
                ].into_iter())
                .chain(tokens.into_iter().flat_map(|t| translate_token(t)))
                .collect()
            ))
        ].into_iter().collect()))
    ].into_iter().collect()
}

fn translate_token(token: Token) -> impl Iterator<Item=TokenTree> {
    match token {
        Token::Text(text) =>
            if text.len() > 0 {call("ui", "label", Literal(Literal::string(&text)).into())} else { vec![].into_iter() },
        Token::Roll => 
            call("ui", "label", call("roll", "to_string", TokenStream::new()).collect()),
        Token::Range => 
            call("ui", "colored_label", [
                Ident(Ident::new("Color32", Span::call_site())),
                Punct(Punct::new(':', Spacing::Joint)),
                Punct(Punct::new(':', Spacing::Alone)),
                Ident(Ident::new("GRAY", Span::call_site())),
                Punct(Punct::new(',', Spacing::Alone)),
                Ident(Ident::new("format", Span::call_site())),
                Punct(Punct::new('!', Spacing::Alone)),
                Group(Group::new(Delimiter::Parenthesis,
                    // TODO clean this mess a bit
                [
                    Literal(Literal::string("({}-{})")),
                    Punct(Punct::new(',', Spacing::Alone)),
                ].into_iter()
                    .chain(iter::once(Ident(Ident::new("this", Span::call_site()))))
                    .chain(iter::once(Punct(Punct::new('.', Spacing::Alone))))
                    .chain(call("roll_range", "start", TokenStream::new()))
                    .chain(iter::once(Punct(Punct::new(',', Spacing::Alone))))
                    .chain(iter::once(Ident(Ident::new("this", Span::call_site()))))
                    .chain(iter::once(Punct(Punct::new('.', Spacing::Alone))))
                    .chain(call("roll_range", "end", TokenStream::new()))
                    .collect())),
            ].into_iter().collect()),
        Token::Bleed    => element("Bleed"),
        Token::Fracture => element("Fracture"),
        Token::Madness  => element("Madness"),
        Token::Void     => element("Void"),
    }
    .chain(iter::once(Punct(Punct::new(';', Spacing::Alone))))
}

fn call(obj: &str, fun: &str, args: TokenStream) -> std::vec::IntoIter<TokenTree> {
    vec![
        Ident(Ident::new(obj, Span::call_site())),
        Punct(Punct::new('.', Spacing::Alone)),
        Ident(Ident::new(fun, Span::call_site())),
        Group(Group::new(Delimiter::Parenthesis, args)),
    ].into_iter()
}

fn element(name: &str) -> std::vec::IntoIter<TokenTree> {
    call("ui", "colored_label", [
        Ident(Ident::new("Element", Span::call_site())),
        Punct(Punct::new(':', Spacing::Joint)),
        Punct(Punct::new(':', Spacing::Alone)),
        Ident(Ident::new(name, Span::call_site())),
        Punct(Punct::new('.', Spacing::Alone)),
        Ident(Ident::new("color", Span::call_site())),
        Group(Group::new(Delimiter::Parenthesis, TokenStream::new())),
        Punct(Punct::new(',', Spacing::Alone)),
        Literal(Literal::string(&name.to_lowercase())),
    ].into_iter().collect())
}
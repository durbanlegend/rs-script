/*[toml]
[dependencies]
quote = "1.0.36"
syn = { version = "2.0.60", features = ["extra-traits", "full", "parsing", "visit", "visit-mut"] }
*/

const INPUT_CODE: &str = stringify! {
    fn foobar() {
      do_something(1, 2, 3);
      do_something_blue(1, 2, 3);
      if some_condition() {
        if other_condition() {
          let a = get_value();
          let b = get_value_blue(a);
        }
      }
    }
};

fn main() {
    use ::quote::ToTokens;
    use ::syn::{visit_mut::*, *};

    let mut code: ItemFn = parse_str(INPUT_CODE).unwrap();

    struct AppendHelloToBlues;
    impl VisitMut for AppendHelloToBlues {
        fn visit_expr_call_mut(self: &'_ mut Self, call: &'_ mut ExprCall) {
            // 1 - subrecurse
            visit_expr_call_mut(self, call);
            // 2 - special case functions whose name ends in `_blue`
            if matches!(
                *call.func,
                Expr::Path(ExprPath { ref path, .. })
                if path.segments.last().unwrap().ident.to_string().ends_with("_blue")
            ) {
                call.args.push(parse_quote!("hello"));
            }
        }

        fn visit_expr_method_call_mut(self: &'_ mut Self, call: &'_ mut ExprMethodCall) {
            // 1 - subrecurse
            visit_expr_method_call_mut(self, call);
            // 2 - special case functions whose name ends in `_blue`
            if call.method.to_string().ends_with("_blue") {
                call.args.push(parse_quote!("hello"));
            }
        }
    }
    AppendHelloToBlues.visit_item_fn_mut(&mut code);
    println!("{}", code.into_token_stream());
}

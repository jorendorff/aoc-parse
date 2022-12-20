#![recursion_limit = "1024"]

use std::fmt::Debug;

use aoc_parse::{parser, prelude::*};

#[track_caller]
fn assert_parse_eq<P, E>(parser: P, s: &str, expected: E)
where
    P: Parser,
    P::Output: PartialEq<E> + Debug,
    E: Debug,
{
    match parser.parse(s) {
        Err(err) => panic!("parse failed: {}", err),
        Ok(val) => assert_eq!(val, expected),
    }
}

#[test]
fn test_rule_set_json() {
    use serde_json::{Value, Number, Map};

    let json = parser!(
        // https://www.rfc-editor.org/rfc/rfc8259#page-5
        rule ws: () = { ' ', '\t', '\r', '\n' }* => ();
        rule value: Value = {
            "null" => Value::Null,
            "false" => Value::Bool(false),
            "true" => Value::Bool(true),
            o:object => Value::Object(o),
            a:array => Value::Array(a),
            n:number => Value::Number(n),
            s:json_string => Value::String(s),
        };
        rule json_string: String =
            '"'
            s:string(char_of( // approximation: any ascii character but " and backslash
                " !#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[]^_`abcdefghijklmnopqrstuvwxyz{|}~"
            )*)
            '"'
            => s;
        rule object: Map<String, Value> =
            ws '{' ws
            members:repeat_sep(member, ws ',' ws)
            ws '}' ws
            => { eprintln!("NUMBER OF MEMBERS: {}", members.len()); members.into_iter().collect()};
        rule member: (String, Value) =
            k:json_string ws ':' ws v:value => (k, v);
        rule array: Vec<Value> =
            ws '[' ws
            elems:repeat_sep(value, ws ',' ws)
            ws ']' ws
            => elems;
        rule number: Number =
            s:string('-'? int frac? exp?) => s.parse::<Number>().unwrap();
        rule int: () = {'0' => (), char_of("123456789") digit* => ()};
        rule frac: () = '.' digit+ => ();
        rule exp: () = {'e', 'E'} i64 => ();

        value
    );

    // Example from the standard (section 13).
    assert_parse_eq(
        &json,
        r#"
            {
              "Image": {
                  "Width":  800,
                  "Height": 600,
                  "Title":  "View from 15th Floor",
                  "Thumbnail": {
                      "Url":    "http://www.example.com/image/481989943",
                      "Height": 125,
                      "Width":  100
                  },
                  "Animated" : false,
                  "IDs": [116, 943, 234, 38793]
                }
            }
          "#,
        serde_json::json!(
            {
                "Image": {
                    "Width":  800,
                    "Height": 600,
                    "Title":  "View from 15th Floor",
                    "Thumbnail": {
                        "Url":    "http://www.example.com/image/481989943",
                        "Height": 125,
                        "Width":  100
                    },
                    "Animated" : false,
                    "IDs": [116, 943, 234, 38793]
                }
            }
        )
    );

    assert_parse_eq(
        &json,
        r#"  [
        {
           "precision": "zip",
           "Latitude":  37.7668,
           "Longitude": -122.3959,
           "Address":   "",
           "City":      "SAN FRANCISCO",
           "State":     "CA",
           "Zip":       "94107",
           "Country":   "US"
        },
        {
           "precision": "zip",
           "Latitude":  37.371991,
           "Longitude": -122.026020,
           "Address":   "",
           "City":      "SUNNYVALE",
           "State":     "CA",
           "Zip":       "94085",
           "Country":   "US"
        }
      ]"#,
        serde_json::json!([
            {
                "precision": "zip",
                "Latitude":  37.7668,
                "Longitude": -122.3959,
                "Address":   "",
                "City":      "SAN FRANCISCO",
                "State":     "CA",
                "Zip":       "94107",
                "Country":   "US"
            },
            {
                "precision": "zip",
                "Latitude":  37.371991,
                "Longitude": -122.026020,
                "Address":   "",
                "City":      "SUNNYVALE",
                "State":     "CA",
                "Zip":       "94085",
                "Country":   "US"
            }
        ])
    );
}

use lib::Json;
use parse_macro::json;

fn main() {
    let x: Vec<Json> = vec![10.into(), 20.into(), "test".into()];

    let nested = json!(
        {
            "a": 10,
            "b": ${10}
        }
    );


    println!("{}", nested.as_json());


    let node = json! {
        {
            "nested": ${nested},
            "string": "test",
            "list": ${x},
            "arbitrary": ${{let var = 10; var}}
        }
    };

    println!("{}", node.as_json());
}

use bake::{Bakeable, bake_fn_once};
use lib::*;
use parse_macro::{self, parse_node, test_bake_macro};

fn main() {
    let test = false;

    let x = parse_node!(
        [
            true,
            ${let x = test_function(10) == 10; x},
            false,
            ${test},
            ${40 > 10},
            true
        ]
    );

    let t = test_bake_macro!();

    let b = parse_node!(true);

    println!("{:?}", x);

    let x = parse_node!([true, true]);

    assert_eq!(parse_node!(true), parse("true").unwrap());

    let x: [u64; 4] = [1, 2, 3, 4];

    println!("{}", x.bake().to_string());

    println!("{}", Ipol {field_a: 10, field_b: 10}.bake().to_string());

}

fn test_function(input: u64) -> u64 {
    input
}

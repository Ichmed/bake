This crate allows you to derive the `Bake` trait with the `bake()` method. `bake()` returns a `TokenStream` that can be used in a proc macro to create an equivalent struct.

Example

```rust
#[derive(Bake)]
struct MyStruct {
    field_a: u64
}

fn main() {
    println!("{}", MyStruct { field_a: 10}.bake().to_string()) 
    // prints: MyStruct { field_a: 10}
}
```

### Quick Feature and Usecase List
- Generate a parsing macro from your existing parsing function
- Use DSLs (Domain Specific Languages) inside hot loops by off-loading parsing to compile time
- Only one implementaion for runtime and compile time parsing
- Interpolate arbitrary Rust into your DSL
- Create injection-safe-by-construction parsers using interpolation

Everything said below about structs is true about named structs, tuple structs, unit structs and all variations of enums (but **not** unions).

## Motivation
The main use case of this crate is to enable the efficient creation of compile time parsing macros.

Assuming you already have a parsing function
```rust
parse_str(input: &str) -> Result<MyType, MyError> {...}
```
and `MyType` derives `Bake`. A simple compile time parsing macro can be written like this.

```rust
#[proc_macro]
fn parse_macro(input: TokenStream) -> TokenStream {
    parse_str(&input.to_string()).unwrap().bake().into()
}
```
(Note that `bake()` returns a `proc_macro2::Tokenstream` and thus requires conversion into a `proc_macro::TokenStream`)

```rust
fn some_func() {
    ...
    let my_struct = parse_macro!(Your syntax here);
    my_struct.my_method();
    ...
}
```

When called the macro will either expand to an instance of `MyType` or will panic which propagates `MyError` to the compiler and stops the compilation. This way the parsed input:

- Is guaranteed to be valid during compile time. 
- Does not need to be parsed during runtime
- Can be used directly and does not need to be unwrapped from a `Result`
- Can benefit from compiler optimizations

## Basic Baking
To add basic baking functionality to your struct simply derive `Bake`. For this to work, all memebers of the struct must already implement `Bake`.

 Note that unlike similar derives like `serde` it is not possible to ignore fields of the struct because all fields need to be baked in order to provide a valid struct. For the same reason all members of your struct must be public, because otherwise they could not be set at the location of the macro call.

 ## Baking private fields
 The code produced by a macro is scoped to the place of the macro invocation, for this reason, while some `TokenStreams` for private may be produced by the `bake()`, they will never actually compile.

 To bake a struct with private fields a constructor function is needed. Note that the `bake()` function can _read_ private fields because it is implemented on the struct itself, it just can not produce them in the resulting TokenStream.

 ```rust
 impl Bake for MyPartialPrivateStruct {
    fn bake(&self) {
        let MyPartialPrivateStruct {
            pub_field,
            priv_field
        } = self;

        // Note that you should fully qualify the module path to the struct
        bake::util::quote!(mycrate::internal::MyPartialPrivateStruct::new(#pub_field, #priv_field))
    }
 }
 ```

 It is not possible to bake a struct with private fields and a private constructor. If you want to work around this you can create a "private-public" constructor like `mycrate::__private::constructor` to make it clear to users of your library that they should not directly call this function.

## Smart Pointers
Baking smart pointers is disabled by default, not because it is not possible, but because it is most likely not what you want.

For example assume we have two `Rc<MyStruct>` pointing at the same instance of `MyStruct`. Calling `bake()` on both of these `Rc`s however will create two separate instances of `MyStruct` at runtime since `bake()` has no way to know about or even refference the other instance.

If you _do_ need smart pointers in your struct you may have to implement bespoke baking logic.

`Box<T>` is excempt from this since it can not be shared.

## Baking Remote Types
Similar to [serde](https://serde.rs/remote-derive.html) you can create a dummy type in order to derive baking logic for a remote type.

```rust
#[derive(Bake)]
#[bake(bake_as(other::crate::Duration))]
pub struct DurationDummy {
    secs: i64,
    nanos: i32,
}

#[derive(Bake)]
pub struct StructWithDuration {
    #[bake_via(DurationDummy)]
    duration: Duration
}
```
This way you can even add [interpolation](#interpolation) to remote types that do not support it.

You can use any type inside `bake_via`, but using a type that is not marked with `bake_as(<other type>)` will most likely error. One big exception to this is unit-types. 

Because no information about the internals of unit types is needed (there is none) remote unit types can be annotated with themselves and will bake just fine (you should still derive `Bake` for your own unit types so people do not have to annotate all uses of them).

Note that this will not work if the unit type is generic.

```rust
#[derive(Bake)]
pub struct StructWithRemoteUnit {
    #[bake_via(RemoteUnit)]
    remote: RemoteUnit
}
```

## Interpolation
[skip motivation](#adding-interpolation)
### Motivation
Interpolation allows you to replace structs with equivalent rust expressions.

For example let's imagine we have a json parser that parses the following static json
```json
{
    "name": "A String",
    "value": 10
}
```
but instead of `10` we want a value that is given as a function parameter. Without interpolation the code would look something like this
```rust
fn wrap_my_number(number: i32) -> Json {
    let mut json = parse_json!{
        {
            "name": "A String",
            "value": 10
        }
    };

    if let Json::Dict(mut map) = json {
        map.insert("value", Json::Number(number));
    }

    json
}
```
which requires us to:
- set `value` to `10` just so we have a valid json
- make json mutable
- awkwardly unwrap the HashMap from the json even though the pattern will always match (also ugly `mut` in the pattern)

Now imagine what the code would look like if the structur was nested further.

With interpolation turned on it looks like this
```rust
fn wrap_my_number(number: i32) -> Json {
    parse_json!{
        {
            "name": "A String",
            "value": ${number}
        }
    }
}
```
Please note that the `${...}` syntax is something you have to implement in your parser, this crate only gives you the framework to bake the interpolation, not to parse them. The macro above will expand to something like this
```rust
Json::Map(
    std::collections::HashMap::from(
        [
            ("name", Json::String("A String".to_owned())),
            ("value", {number}.into())
        ]
    )
)
```
As you can see this requires a `From<i32>` impl for `Json` which you would most likely implement anyway. Because of the blanket implementation `impl<T> From<T> for T` it is always valid to put a value of the type that would be expected in that place by the normal parser.
```rust
fn wrap_my_node(node: Json) -> Json {
    parse_json!{
        {
            "name": "Look! I wrapped a node!",
            "value": ${node}
        }
    }
}
```

### Adding Interpolation
Adding interpolation to a struct is as simple as annotatig it with `#[bake(interpolation)]`
```rust
#[derive(Bake, Debug, PartialEq)]
#[bake(interpolate)]
pub enum Json {
    Number(i64),
    Boolean(bool),
    String(String),
    List(Vec<Json>),
    Dict(HashMap<String, Box<Json>>)
}
```
Interacting with the struct becomes a bit trickier though: For the user of your crate not much changes apart from being able to interpolate, but you now have to make sure that all your code works whether you are interpolating or not.

### The 'macro' feature
Marking any type as interpolated implicitly adds the 'macro' feature to your crate. Interpolation is only available when the crate is imported with this feature enabled, otherwise it is assumed that all fields are just plain types.

When the macro feature is turned on all interpolated fields `field: T` become `field: Interpolatable<T>` instead. `Interpolatable<T>` is an enum with two variants: 
- `Actual(T)` represents an actual value of type `T` and gets baked the same way `T` would
- `Interpolation(TokenTree)` represents a rust block that ***should*** evaluate to a type the implements `Into<T>` and gets backed as `{/*TokenTree here*/}.into()`

Creating a `Interpolatable::<T>::Interpolation` that can not be converted into a `T` with `into()` will produce a compiler error when calling the macro.

### Adjusting code
You will have the following changes to your code:
- parsing functions that _may_ return an interpolated value need there return type from `T` to `Interpolatable<T>`
- Struct constructors from parsing functions need to call `.fit()?` on all fields. This will convert betweeen `T` and `Interpolatable<T>` as needed.
- Your parsing errors need to implement `From<bake::RuntimeInterpolationError>` in order for `.fit()?` to work
- Guard all functions that need to work on a raw `T` by
  - using `fit()?` or `force_fit()`
    ```rust
    ...
    _ => match tokens {
        "true" => Ok(Json::Boolean(true.force_fit()).fit()?),
        "false" => Ok(Json::Boolean(false.force_fit()).fit()?),
        _ => Err(NodeError::Parsing)
    },
    ...
    ```
  - guarding them behind a `#[cfg(not(feature = "macro"))]` so they can not be called during macro-parsing

    ```rust  
    #[cfg(not(feature = "macro"))]
    impl Json {
        pub fn truthyness(&self) -> bool{
            match self {
                Json::Number(x) => *x != 0,
                Json::Boolean(x) => *x,
                Json::String(x) => x.len() > 0,
                Json::List(x) => !x.is_empty(),
                Json::Dict(x) => !x.is_empty()
            }
        }
    }
    ```
    Note that the code your macro produces may still call these methods, you just can not call them _inside_ of your proc_macro

`fit()?` will be replaced with just `?` as soon as `Try` is stabilized.

### Runtime Interpolation
Trying to interpolate during runtime is always an error, for this reason `fit()` returns a `Result` that is always `Ok` unless you try to convert from `Interpolatable::<T>::Interpolation` to a `T`. `force_fit()` is just short for `fit().expect("Interpolated during runtime")` and can be used if you know for sure that you have an `Actual(T)` or `T` like in `Json::Boolean(false.force_fit())`.

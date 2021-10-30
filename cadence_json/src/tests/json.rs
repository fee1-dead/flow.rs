use crate::ValueOwned;

macro_rules! parse_tests {
    ($(fn $name:ident() {$($tt: tt)*} should_be $exp:expr)+) => {$(
        #[test]
        fn $name() {
            let js = concat!("{", stringify!($($tt)*), "}");

            let found: ValueOwned = serde_json::from_str(js).expect("Failed to parse JSON");
            let expected: ValueOwned = $exp;

            assert_eq!(expected, found);
        })+
    };
}

parse_tests! {
    fn void() {
        "type": "Void"
    } should_be ValueOwned::Void

    fn string() {
        "type": "String",
        "value": "Hello World"
    } should_be ValueOwned::String("Hello World".into())

    fn deeply_nested_array() {
        "type": "Array",
        "value": [
            {
                "type": "Array",
                "value": [{
                    "type": "Array",
                    "value": [
                        {
                            "type": "Array",
                            "value": [{
                                "type": "Array",
                                "value": [
                                    {
                                        "type": "Array",
                                        "value": [{
                                            "type": "Array",
                                            "value": [
                                                {
                                                    "type": "Array",
                                                    "value": []
                                                }
                                            ]
                                        }]
                                    }
                                ]
                            }]
                        }
                    ]
                }]
            }
        ]
    } should_be ValueOwned::Array(vec![
        ValueOwned::Array(vec![
            ValueOwned::Array(vec![
                ValueOwned::Array(vec![
                    ValueOwned::Array(vec![
                        ValueOwned::Array(vec![
                            ValueOwned::Array(vec![
                                ValueOwned::Array(vec![])
                            ])
                        ])
                    ])
                ])
            ])
        ])
    ])
}

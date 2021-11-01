use num_bigint::{BigInt, BigUint, Sign};

use crate::*;

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

    fn optional_none() {
        "type": "Optional",
        "value": null
    } should_be ValueOwned::Optional(None)

    fn bool() {
        "type": "Bool",
        "value": true
    } should_be ValueOwned::Bool(true)

    fn string() {
        "type": "String",
        "value": "Hello World"
    } should_be ValueOwned::String("Hello World".into())

    fn address() {
        "type": "Address",
        "value": "0x1234"
    } should_be ValueOwned::Address(AddressOwned { data: [0x12, 0x34].into() })

    fn uint8() {
        "type": "UInt8",
        "value": "123"
    } should_be ValueOwned::UInt8(123)

    fn uint() {
        "type": "UInt",
        "value": "115792089237316195423570985008687907853269984665640564039457584007913129639936"
    } should_be ValueOwned::UInt(BigUint::from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 1]))

    fn fixed_point() {
        "type": "Fix64",
        "value": "12.3"
    } should_be ValueOwned::Fix64(Fix64::from_raw(1230000000))

    fn array() {
        "type": "Array",
        "value": [
          {
            "type": "Int16",
            "value": "123"
          },
          {
            "type": "String",
            "value": "test"
          },
          {
            "type": "Bool",
            "value": true
          }
        ]
    } should_be ValueOwned::Array(vec![ ValueOwned::Int16(123), ValueOwned::String("test".into()), ValueOwned::Bool(true) ])

    fn dictionary() {
        "type": "Dictionary",
        "value": [
          {
            "key": {
              "type": "UInt8",
              "value": "123"
            },
            "value": {
              "type": "String",
              "value": "test"
            }
          }
        ]
    } should_be ValueOwned::Dictionary(vec![ EntryOwned {
        key: ValueOwned::UInt8(123),
        value: ValueOwned::String("test".into())
    }])

    fn composite() {
        "type": "Resource",
        "value": {
          "id": "0x3.GreatContract.GreatNFT",
          "fields": [
            {
              "name": "power",
              "value": {"type": "Int", "value": "1"}
            }
          ]
        }
    } should_be ValueOwned::Resource(CompositeOwned {
        id: "0x3.GreatContract.GreatNFT".into(),
        fields: vec![
            CompositeFieldOwned {
                name: "power".into(),
                value: ValueOwned::Int(BigInt::new(Sign::Plus, vec![1]))
            }
        ]
    })

    fn path() {
        "type": "Path",
        "value": {
          "domain": "storage",
          "identifier": "flowTokenVault"
        }
    } should_be ValueOwned::Path(PathOwned {
        domain: PathDomain::Storage,
        identifier: "flowTokenVault".into(),
    })

    fn ty() {
        "type": "Type",
        "value": {
          "staticType": "Int"
        }
    } should_be ValueOwned::Type("Int".into())

    fn capability() {
        "type": "Capability",
        "value": {
          "path": "/public/someInteger",
          "address": "0x01",
          "borrowType": "Int",
        }
    } should_be ValueOwned::Capability(CapabilityOwned {
        path: "/public/someInteger".into(),
        address: AddressOwned { data: [0x01].into() },
        borrow_type: "Int".into(),
    })

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

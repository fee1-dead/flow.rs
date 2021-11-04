macro_rules! algorithms_impl {
    ($(
        $(#[$algometa:meta])*
        $algo:ident {
            $(
                $(#[$meta:meta])*
                $name:ident = ($code:expr, $algoname:expr)
            ),+$(,)?
        }
    )+) => {
        mod private {
            pub trait Sealed {}
        }
        $(
            $(#[$algometa])*
            pub trait $algo: private::Sealed {
                /// The code of the algorithm.
                const CODE: u32;

                /// The name of the algorithm.
                const NAME: &'static str;
            }
            $(
                $(#[$meta])*
                pub struct $name;
                impl private::Sealed for $name {}
                impl $algo for $name {
                    const CODE: u32 = $code;
                    const NAME: &'static str = $algoname;
                }
            )+
        )+
    };
}
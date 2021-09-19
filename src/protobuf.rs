include!("../proto_defs.rs");

macro_rules! import_pkgs {
    ($($pkg:ident),+$(,)?) => {
        $(
            pub mod $pkg {
                include!(concat!(env!("OUT_DIR"), "/flow.", stringify!($pkg), ".rs"));
            }
        )+
    };
}

proto_pkgs! {
    import_pkgs;
}
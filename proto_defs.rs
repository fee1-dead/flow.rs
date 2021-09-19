#[macro_export]
macro_rules! proto_pkgs {
    ($callback: ident $($semi:tt)?) => {
        $callback !( // Add packages here
            access,
            entities,
            execution,
        )$($semi)?
    };
}
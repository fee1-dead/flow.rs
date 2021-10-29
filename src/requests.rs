pub(crate) mod private {
    pub trait Sealed {}
}

/// A callable request of the Flow Access API.
pub trait FlowRequest<Response>: private::Sealed {
    /// The path of the request.
    ///
    /// formatted as "/"({package} ".")? {service}"/" {method}.
    const PATH: &'static str;
}

/// Create and populate a mount.
///
/// ```ignore
/// let router = router!("/"       => index,
///                      "/:query" => queryHandler)
/// ```
///
/// Is equivalent to:
///
/// ```ignore
///    let mut mount = Mount::new();
///    mount.mount("/", index);
///    mount.mount("/:query", queryHandler);
/// ```
#[macro_export]
macro_rules! mount {
    ($($mountpoint:expr => $handler:expr),+ $(,)*) => ( {
        let mut mount = $crate::Mount::new();
        $(mount.mount($mountpoint, $handler);)*
        mount
    });
}


#[cfg(test)]
mod tests {
    use iron::{Response, Request, IronResult};

    #[test]
    fn methods() {
        fn handler(_: &mut Request) -> IronResult<Response> {Ok(Response::new())}
        let _ = mount!("/" => handler,
                       "/foo" => handler);
    }
}

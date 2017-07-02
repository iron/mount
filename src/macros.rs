/// Create a set of mounts.
///
/// E.g. if you had a directory to serve images from:
///
/// ```ignore
/// let mount = mount!("/images" => handler_for_image_dir);
/// ```
///
/// Is equivalent to:
///
/// ```ignore
/// let mut mount = Mount::new();
/// mount.mount("/images", handler_for_image_dir);
/// ```
///
/// You may provide multiple mappings, comma-delimited.
#[macro_export]
macro_rules! mount {
    ($($path:expr => $handler:expr),+ $(,)*) => ({
        let mut mount = $crate::Mount::new();
        $(mount.mount($path, $handler);)*
        mount
    });
}

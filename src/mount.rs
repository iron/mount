use std::error::Error;
use iron::{Handler, Response, Request, IronResult, IronError, Url, status};
use iron::typemap;
use sequence_trie::SequenceTrie;
use std::fmt;

/// Exposes the original, unmodified path to be stored in `Request::extensions`.
#[derive(Debug, Copy)]
pub struct OriginalUrl;
impl typemap::Key for OriginalUrl { type Value = Url; }

/// Exposes the mounting path, so a request can know its own relative address.
#[derive(Debug, Copy)]
pub struct VirtualRoot;
impl typemap::Key for VirtualRoot { type Value = Url; }

/// `Mount` is a simple mounting middleware.
///
/// Mounting allows you to install a handler on a route and have it receive requests as if they
/// are relative to that route. For example, a handler mounted on `/foo/` will receive
/// requests like `/foo/bar` as if they are just `/bar`. Iron's mounting middleware allows
/// you to specify multiple mountings using one middleware instance. Requests that pass through
/// the mounting middleware are passed along to the mounted handler that best matches the request's
/// path. `Request::url` is modified so that requests appear to be relative to the mounted handler's route.
///
/// Mounted handlers may also access the *original* URL by requesting the `OriginalUrl` key
/// from `Request::extensions`.
pub struct Mount {
    inner: SequenceTrie<String, Match>
}

struct Match {
    handler: Box<Handler>,
    length: usize
}

/// The error returned by `Mount` when a request doesn't match any mounted handlers.
#[derive(Debug)]
pub struct NoMatch;

impl Error for NoMatch {
    fn description(&self) -> &'static str { "No Match" }
}

impl fmt::Display for NoMatch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.description())
    }
}

impl Mount {
    /// Creates a new instance of `Mount`.
    pub fn new() -> Mount {
        Mount {
            inner: SequenceTrie::new()
        }
    }

    /// Mounts a given `Handler` onto a route.
    ///
    /// This method may be called multiple times with different routes.
    /// For a given request, the *most specific* handler will be selected.
    ///
    /// Existing handlers on the same route will be overwritten.
    pub fn on<H: Handler>(&mut self, route: &str, handler: H) -> &mut Mount {
        // Parse the route into a list of strings. The unwrap is safe because strs are UTF-8.
        let key = Path::new(route).str_components()
            .map(|s| s.unwrap().to_string()).collect::<Vec<_>>();

        // Insert a match struct into the trie.
        self.inner.insert(key.as_slice(), Match {
            handler: Box::new(handler) as Box<Handler>,
            length: key.len()
        });
        self
    }

    /// The old way to mount handlers.
    #[deprecated = "use .on instead"]
    pub fn mount<H: Handler>(&mut self, route: &str, handler: H) -> &mut Mount {
        self.on(route, handler)
    }
}

impl Handler for Mount {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let original = req.url.path.clone();

        // If present, remove the trailing empty string (which represents a trailing slash).
        // If it isn't removed the path will never match anything, because
        // Path::str_components ignores trailing slashes and will never create routes
        // ending in "".
        let mut root = original.as_slice();
        while root.last().map(|s| &**s) == Some("") {
            root = &root[..root.len() - 1];
        }

        // Find the matching handler.
        let matched = match self.inner.get_ancestor(root) {
            Some(matched) => matched,
            None => return Err(IronError::new(NoMatch, status::NotFound))
        };

        // We have a match, so fire off the child.
        // If another mount middleware hasn't already, insert the unmodified url
        // into the extensions as the "original url".
        let is_outer_mount = !req.extensions.contains::<OriginalUrl>();
        if is_outer_mount {
            let mut root_url = req.url.clone();
            root_url.path = root.to_vec();

            req.extensions.insert::<OriginalUrl>(req.url.clone());
            req.extensions.insert::<VirtualRoot>(root_url);
        } else {
            req.extensions.get_mut::<VirtualRoot>().map(|old| {
                old.path.push_all(root);
            });
        }

        // Remove the prefix from the request's path before passing it to the mounted
        // handler. If the prefix is entirely removed and no trailing slash was present,
        // the new path will be the empty list.
        //
        // For the purposes of redirection, conveying that the path did not include
        // a trailing slash is more important than providing a non-empty list.
        req.url.path = req.url.path.as_slice()[matched.length..].to_vec();

        let res = matched.handler.handle(req);

        // Reverse the URL munging, for future middleware.
        req.url.path = original.clone();

        // If this mount middleware is the outermost mount middleware,
        // remove the original url from the extensions map to prevent leakage.
        if is_outer_mount {
            req.extensions.remove::<OriginalUrl>();
            req.extensions.remove::<VirtualRoot>();
        } else {
            req.extensions.get_mut::<VirtualRoot>().map(|old| {
                let old_len = old.path.len();
                old.path.truncate(old_len - root.len());
            });
        }

        res
    }
}


use iron::{Handler, Response, Request, IronResult, Url, Error};
use typemap::Assoc;

use self::matcher::Matcher;

mod matcher;

/// Exposes the original, unmodified path to be stored in `Request::extensions`.
pub struct OriginalUrl;
impl Assoc<Url> for OriginalUrl {}

/// `Mount` is a simple mounting middleware.
///
/// `Mount` allows you to mount other middleware within a larger chain.
///
/// Mounted middleware will receive a modified version of the request
/// url with the mount pattern removed from the start of the url.
pub struct Mount {
    inner: Matcher<Match>
}

struct Match {
    handler: Box<Handler + Send + Sync>,
    length: uint
}

/// The error returned by Mount when there is no match.
#[deriving(Show)]
pub struct NoMatch;

impl Error for NoMatch {
    fn name(&self) -> &'static str { "No Match" }
}

impl Mount {
    /// Creates a new instance of `Mount`
    pub fn new() -> Mount {
        Mount {
            inner: Matcher::new()
        }
    }

    /// Mount a given `Handler` onto this route.
    pub fn mount<H: Handler>(&mut self, route: &str, handler: H) -> &mut Mount {
        let key: Vec<String> = Path::new(route).str_components().map(|s| s.unwrap().to_string()).collect();
        self.inner.insert(key.clone(), Match { handler: box handler as Box<Handler + Send + Sync>, length: key.len() });
        self
    }
}

impl Handler for Mount {
    fn call(&self, req: &mut Request) -> IronResult<Response> {
        match self.inner.find(req.url.path.as_slice()) {
            Some(matched) => {
                    // We have a match, so fire off the child.
                    // If another mount middleware hasn't already, insert the unmodified url
                    // into the extensions as the "original url".
                    let is_outer_mount = !req.extensions.contains::<OriginalUrl, Url>();
                    if is_outer_mount {
                        req.extensions.insert::<OriginalUrl, Url>(req.url.clone());
                    }

                    // Remove the prefix from the request's path before passing it to the mounted middleware.
                    // If the prefix is entirely removed and no trailing slash was present, the new path
                    // will be the empty list. For the purposes of redirection, conveying that the path
                    // did not include a trailing slash is more important than providing a non-empty list.
                    req.url.path = req.url.path.as_slice().slice_from(matched.length).to_vec();

                    let res = matched.handler.call(req);

                    // Reverse the URL munging, for future middleware.
                    req.url = match req.extensions.find::<OriginalUrl, Url>() {
                        Some(original) => original.clone(),
                        None => fail!("OriginalUrl unexpectedly removed from req.extensions.")
                    };

                    // If this middleware is the outermost mount middleware,
                    // remove the original url from the extensions map to prevent leakage.
                    if is_outer_mount {
                        req.extensions.remove::<OriginalUrl, Url>();
                    }

                    res
            },
            None => Err(box NoMatch as Box<Error>)
        }
    }
}


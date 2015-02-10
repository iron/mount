pub use iron::prelude::*;
pub use iron::Handler;
pub use iron::{status, method};

// FIXME(reem): Write tests for OriginalUrl and VirtualRoot
pub use {Mount, OriginalUrl, VirtualRoot, NoMatch};

pub fn at(mount: &Mount, url: &str) -> Result<String, IronError> {
    use std::old_io::util::NullReader;
    use itest::mock::request;
    use iron::Url;

    let url = Url::parse(&format!("http://localhost:3000{}", url)).unwrap();
    let rdr = &mut NullReader;
    let mut req = request::new(method::Get, url, rdr);

    mount.handle(&mut req).map(|res| {
       res.body.unwrap_or(Box::new(NullReader)).read_to_string().unwrap()
    })
}

describe! mount {
    before_each {
        let mut mount = Mount::new();

        mount.on("/hello", |_: &mut Request| {
            Ok(Response::with((status::Ok, "hello")))
        });

        mount.on("/intercept", |_: &mut Request| {
            Ok(Response::with((status::Ok, "intercepted")))
        });

        mount.on("/trailing_slashes///", |_: &mut Request| {
            Ok(Response::with((status::Ok, "trailing slashes")))
        });

        mount.on("/trailing_slash/", |_: &mut Request| {
            Ok(Response::with((status::Ok, "trailing slash")))
        });
    }

    it "should mount handlers" {
        assert_eq!(&*at(&mount, "/hello").unwrap(), "hello");
        assert_eq!(&*at(&mount, "/hello/and/more").unwrap(), "hello");

        assert_eq!(&*at(&mount, "/intercept").unwrap(), "intercepted");
        assert_eq!(&*at(&mount, "/intercept/with/more").unwrap(), "intercepted");
    }

    it "should work with trailing slashes" {
        assert_eq!(&*at(&mount, "/hello/").unwrap(), "hello");
        assert_eq!(&*at(&mount, "/hello//").unwrap(), "hello");
        assert_eq!(&*at(&mount, "/hello//and/more").unwrap(), "hello");

        assert_eq!(&*at(&mount, "/trailing_slash").unwrap(), "trailing slash");
        assert_eq!(&*at(&mount, "/trailing_slash///").unwrap(), "trailing slash");
        assert_eq!(&*at(&mount, "/trailing_slash/with_more").unwrap(), "trailing slash");
        assert_eq!(&*at(&mount, "/trailing_slash//crazy/with_more").unwrap(), "trailing slash");

        assert_eq!(&*at(&mount, "/trailing_slashes").unwrap(), "trailing slashes");
        assert_eq!(&*at(&mount, "/trailing_slashes/").unwrap(), "trailing slashes");
        assert_eq!(&*at(&mount, "/trailing_slashes///").unwrap(), "trailing slashes");
        assert_eq!(
            &*at(&mount, "/trailing_slashes///with_extra/crazy").unwrap(),
            "trailing slashes"
        );
    }

    it "should throw when no match is found" {
        let err = at(&mount, "/notfound").unwrap_err();

        assert_eq!(err.response.status, Some(status::NotFound));
        err.error.downcast::<NoMatch>().unwrap();
    }
}


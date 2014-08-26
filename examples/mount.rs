extern crate iron;
extern crate mount;

use std::io::net::ip::Ipv4Addr;

use iron::status;
use iron::{Iron, Request, Response, IronResult};
use mount::Mount;

fn intercept(_: &mut Request) -> IronResult<Response> { Ok(Response::with(status::Ok, "Blocked!")) }

fn send_hello(_: &mut Request) -> IronResult<Response> { Ok(Response::with(status::Ok, "Hello!")) }

fn main() {
    let mut mount = Mount::new();
    mount.mount("/blocked/", intercept).mount("/", send_hello);

    Iron::new(mount).listen(Ipv4Addr(127, 0, 0, 1), 3000);
}


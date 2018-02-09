#![feature(plugin, decl_macro)]
#![plugin(rocket_codegen)]

extern crate libc;
extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;
extern crate serde_json;

use libc::{c_double, c_int};

use rocket_contrib::Json;

#[derive(Serialize)]
struct LoadAvg {
    last: f64,
    last5: f64,
    last15: f64
}

extern {
    fn getloadavg(load_avg: *mut c_double, load_avg_len: c_int);
}


impl LoadAvg {
    fn new() -> LoadAvg {
        let load_averages: [f64; 3] = unsafe {
            let mut lavgs: [c_double; 3] = [0f64, 0f64, 0f64];
            getloadavg(lavgs.as_mut_ptr(), 3);
            lavgs
        };

        LoadAvg {
            last: load_averages[0],
            last5: load_averages[1],
            last15: load_averages[2]
        }
    }
}

#[get("/loadavg")]
fn loadavg() -> Json<LoadAvg> {
    Json(LoadAvg::new())
}

fn main() {
    rocket::ignite()
        .mount("/", routes![loadavg])
        .launch();
}

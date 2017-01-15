#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate libc;
extern crate rocket;

use libc::{c_double, c_int};


#[derive(Debug)]
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
fn loadavg() -> String {
    format!("{:?}", LoadAvg::new())
}

fn main() {
    rocket::ignite()
        .mount("/", routes![loadavg])
        .launch();
}

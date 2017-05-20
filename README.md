# Building high performance REST APIs with Rust and Rocket

This project is done as part of a workshop to show how to build APIs using [Rocket](https://rocket.rs). It was originally written in Spanish for the RustMX meetup as you can find it in `README.es.md`.

The goal of this project is to show the fundamental concepts needed to implement a REST API using Rust and Rocket, highlighting some specific details over the way.

The slidedeck for this workshop is [available in Spanish](http://slides.com/sebasmagri/construyendo-servicios-web-de-alto-rendimiento-con-rust-y-rocket). However, this document describes the implementation in more detail.

## Objectives

The goal of this API will be to allow clients to query the load average of a host through a single endpoint.

Load average is an abstraction of how busy a host has been in the last minute, the last 5 minutes and the last 15 minutes. The values for each timeframe are a relation between the system's capacity to process tasks and the amount of tasks to be processed.

Clients will query the load average issuing a `GET` request to a `/loadavg` endpoint, and they will get a JSON answer as follows:

    {
        'last': 0.7,
        'last5': 1.1,
        'last15': 0.8
    }

## Preparing the environment

Rocket *still* requires Rust *Nightly* because of some features that have not yet landed in a stable release of the compiler. Fortunately, [rustup](https://rustup.rs/) makes it really easy to install and manage multiple Rust compiler releases. To install Rust Nightly, we can run the official rustup script:

    $ curl https://sh.rustup.rs -sSf | sh

This method _just works_ for  UNIX environments. If you're working on Windows you should follow [other installation methods](https://github.com/rust-lang-nursery/rustup.rs/#other-installation-methods).

By default, rustup installs the stable toolchain. Then, we need to install the nightly toolchain with:

    $ rustup install nightly-2017-05-18

If your Rocket application stops working after an update, you should update your toolchain as well:

    $ rustup update

## Creating the new project

In the Rust world, the project, dependencies and build management is done using *Cargo*. Cargo automates a lot of tasks and you will be definitely using it really often while working with Rust.

To generate the initial files structure of our application we can run:

    $ cargo new loadavg-api --bin
    $ cd loadavg-api/

Now, we must set the project to use the nightly toolchain:

    $ rustup override set nightly-2017-01-25

## Rocket installation

Now that we have our project in place, lets add *Rocket* to its dependencies.

`cargo` tracks dependencies in a `Cargo.toml` file found the project root. We must use the `[dependencies]` section on this file to define which *crates* are going to be used by our project. By default, those crates are fetched from the central community repo at [crates.io](https://crates.io/). Thus, we add `rocket` and `rocket_codegen` to our dependencies. The latter includes code generation tools and it makes it a lot easier to implement APIs.

    [dependencies]
    rocket = "0.1.6"
    rocket_codegen = "0.1.6"

The next time we run `cargo build` or `cargo run`, cargo will automatically find, fetch and build all of the dependencies.

## Building the API

### Initial modelling

As a first step, lets do a model of the data that will be handled by our API. Having a strong functional programming influence, Rust uses *data types* for this.

#### Data Types

Rust allows the definition of new data types by using `struct`s. Then, would we need an abstraction of the load average, we could implement it as follows:

    #[derive(Debug)]
    struct LoadAvg {
        last: f64,  // last minute load average
        last5: f64,  // last 5 minutes load average
        last15: f64  // last 15 minutes load average
    }

Here we are creating a `LoadAvg` `struct` with 3 *fields*, each one of those has a `f64` data type, the Rust primitive data type for 64 bits floating point numbers. This struct is by itself a new data type which abstracts the concept of load average. If we look closely at the JSON response that clients should be getting, we will find `LoadAvg` to be pretty similar.

Above the definition of our `LoadAvg` struct, we can find `#[derive(Debug)]`. This is a way in which Rust _implements_ a `trait`. The `trait` describes certain specific behaviours of a data type. In this specific case, to aid with debugging, we are adding `LoadAvg` the necessary behaviour to be able to print an instance of it to the standard output by using the `{:?}` format specifier. This way we can get a detailed representation of the data type:

    println!("{:?}", load_avg);
    ...
    LoadAvg { last: 0.9, last5: 1.5, last15: 1.8 }

We can now add our new data type to the `src/main.rs` file and go on.

#### Data type behaviour

Rust `struct`s are not static structures. Rust actually allows is to model a data type behaviour by using *methods*, in a similar way to the object oriented programming classes. For example, to add a _constructor_ to our `LoadAvg` data type, we can use an `impl` block:

    impl LoadAvg {
        fn new() -> LoadAvg {
            // Placeholder
            LoadAvg {
                last: 0.9,
                last5: 1.5,
                last15: 1.8
            }
        }
    }

We will be able to use the `new` method onwards to create *instances* of this data type. For example, in our `main` function in `src/main.rs`, we could use:

    fn main() {
        let load_avg = LoadAvg::new();
        println!("{:?}", load_avg);
    }

##### Getting real load average data

This particular section is not implemented in the workshop because of time constraints, but it is documented in detail here to show how to integrate C standard library functions in Rust.

Until now, we have been using placeholder values for the fields of `LoadAvg`. However, one would like `LoadAvg::new()` to return an instance with the current load average values.

The recommended way to get the system's load average is using the `getloadavg` function from `libc`, the C standard library. However, this function is implemented in *C*, and C do not give us the safeguards that Rust offers. Even so, it's quite simple to integrate it in our Rust code. We must indicate it's an _external_ function, and it's _unsafe_.

First of all, lets add a reference to `libc` in our project's `[dependencies]` in the `Cargo.toml` file:

    libc = "*"

Then, we can reference this _crate_ in our source code, at the top of `src/main.rs`:

    extern crate libc;

This allow us to use any of the functions defined in the [libc](https://doc.rust-lang.org/libc/x86_64-unknown-linux-gnu/libc/) crate in our project.

Now, if we look at the [getloadavg function signature in C](https://linux.die.net/man/3/getloadavg), we will see that the first parameter is a pointer to an array of `double` values, and the second one is an `int`:

    # This is C code
    int getloadavg(double loadavg[], int nelem);

However, neither `double` nor `int` are present among the primitive Rust data types, and we need to find an implementation of those data types for Rust. Fortunately, we can find it as `c_double` and `c_int` in the `libc` crate, so we _use_ them in our code:

    use libc::{c_double, c_int};

Then, we are able to add a reference to this function in our Rust code:

    extern {
        fn getloadavg(load_avg: *mut c_double, load_avg_len: c_int);
    }

As we can see, this function will take a _mutable_ `c_double` pointer to the first element of the output array, and a `c_int` for the count of elements.

Now we're able to call `getloadavg`:

    let load_averages: [f64; 3] = unsafe {
        let mut lavgs: [c_double; 3] = [0f64, 0f64, 0f64];
        getloadavg(lavgs.as_mut_ptr(), 3);
        lavgs
    };

This way, our `LoadAvg::new` _constructor_ can be:

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

### API implementation

#### The /loadavg endpoint

According to the initial requirement, we need a `/loadavg` _endpoint_ that will handle `GET` requests and will respond with the load average in JSON.

To accomplish this, Rocket maps a _route_ and a set of validation conditions to a function that will _handle_ the input data and generate a response. The validations are concisely expressed through *attributes* in the functions. This attribute is used to define the request _method_, parameters and constraints of a specific endpoint.

With this in mind, the responsibility of our _handler_ will be to create a new instance of `LoadAvg` y return its value in JSON.

First, we add the necessary references to our `src/main.rs` for the Rocket tools. At the top of the main file, we must add some directives to tell the compiler that we'll be using some custom features:

    #![feature(plugin)]
    #![plugin(rocket_codegen)]

    extern crate rocket;

Next, lets implement an initial handler for our endpoint:

    #[get("/loadavg")]
    fn loadavg() -> String {
        format!("{:?}", LoadAvg::new())
    }

The handler definition starts with an *attribute* in which we define the request method, the route and the parameters of an endpoint. `#[get("/loadavg")]` indicates that the following function will respond only to `GET` requests to the `/loadavg` path, and will not take any parameter.

After the _attribute_, a function is defined to handle the matching requests. The function's return data type must implement the `Responder` trait, that defines how a data type is transformed into a HTTP response.

We have not had to implement `Responder` anywhere by ourselves because Rocket already implements it for a bunch of standard data types.

#### Mounting the /loadavg endpoint

Our handler is not yet available for clients. It must be _mounted_ first when the application starts. Then, the Rocket's Web server must be started in the `main` function of our project. It's a funny launch sequence. After the engine _ignition_, the `mount` function allows to provide a set of _routes_ by using the `routes!` macro. After all routes has been mounted, you can then _launch_ the _rocket_:

    fn main() {
        rocket::ignite()
            .mount("/", routes![loadavg])
            .launch();
    }

Now we can run our application using `cargo run`:

    🚀  Rocket has launched from http://localhost:8000...

However, if we query the endpoint at `http://localhost:8000/loadavg`, we'll realize the content of the response is not in JSON yet. But this is going to change pretty soon.

#### Serializing the response as JSON

Ultimately, we need to make sure the response is properly formatted according to the JSON initial specification, and that the adequate headers are set for clients to be able to process the response properly. This may sound complicated, but Rocket provides tools to handle JSON easily in its `contrib` crate. The `rocket_contrib::JSON` data type allows us to wrap a _serializable_ data type and make it the handler output type, so it will handle all the specific details automatically.

The `JSON` data type requires some additional dependencies to work. More specifically, it uses the `serde` crate, which is probably the most used for _serialization_ and _deserialization_ in Rust. Lets add the needed bits to our `[dependencies]` section so it looks as follows:

    [dependencies]
    libc = "*"
    rocket = "0.1.6"
    rocket_codegen = "0.1.6"
    rocket_contrib = { version = "0.1.6", features = ["json"] }
    serde = "0.8"
    serde_json = "0.8"
    serde_derive = "0.8"

Then, we need to add the crates references at the top of our `src/main.rs` file:

    extern crate serde_json;
    #[macro_use] extern crate rocket_contrib;
    #[macro_use] extern crate serde_derive;
    
    use rocket_contrib::JSON;

At this point, we only need to make sure that our response data type can be correctly serialized as JSON. Given that `LoadAvg` is pretty simple, and all of its fields can be easily translated to its JSON counterparts, we can use `#[derive()]` to automatically implement `serde`'s `Serialize` trait:

    #[derive(Serialize)]
    struct LoadAvg {
        last: f64,
        last5: f64,
        last15: f64
    }

We've removed the `Debug` trait as well since we wont be using it anymore.

By giving our data type the super power to be transformed to `JSON`, we can refactor our handler to return `rocket_contrib::JSON`:

    #[get("/loadavg")]
    fn loadavg() -> JSON<LoadAvg> {
        JSON(LoadAvg::new())
    }

Finally, we can run the application again using `cargo run` and check how the response for the `/loadavg` endpoint is correctly formatted.

## Final references

* https://doc.rust-lang.org/stable/book/
* https://rocket.rs/guide/requests/#json
* https://github.com/SergioBenitez/Rocket/tree/v0.1.6/examples/json

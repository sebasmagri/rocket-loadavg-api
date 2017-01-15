# Construyendo APIs de alto rendimiento con Rust y Rocket

Este proyecto muestra los fundamentos para la implementación de un API REST en el lenguaje de programación Rust, haciendo uso del Framework Web Rocket.

También está disponible una [presentación introductoria](http://slides.com/sebasmagri/construyendo-servicios-web-de-alto-rendimiento-con-rust-y-rocket) como acompañante de este proyecto que es recomendable repasar antes de iniciar.

## Objetivos

El objetivo de este API será consultar la carga promedio de un sistema a través de un único endpoint.

La carga promedio expresa qué tan ocupado ha estado un sistema *procesando* tareas, y se expresa generalmente en forma de 3 valores; para el último minuto, para los últimos 5 minutos y para los últimos 15 minutos. La magnitud de cada valor es una aproximación a la relación entre la capacidad de procesar tareas y la cantidad de tareas en procesamiento durante ese tiempo.

Los clientes consultarán la carga del sistema con una solicitud `GET` a un endpoint `/loadavg`, y recibirán una respuesta en `JSON` con la siguiente forma:

    {
        'last': 0.7,
        'last5': 1.1,
        'last15': 0.8
    }

## Preparación del ambiente de trabajo

Rocket *aún* requiere el uso de la versión *nightly* o de desarrollo del compilador debido a que hace uso de algunas características del lenguaje que aún no están disponibles en las versiones estables. Afortunadamente, [rustup](https://rustup.rs/) hace que sea muy fácil instalar y manejar cualquier versión de Rust en nuestros ambientes de desarrollo. Para instalar Rust, ejecutamos el script oficial:

    $ curl https://sh.rustup.rs -sSf | sh

Este método funciona para ambientes UNIX, si estás trabajando en Windows puedes usar [otros métodos de instalación](https://github.com/rust-lang-nursery/rustup.rs/#other-installation-methods).

`rustup` instala por defecto el toolchain estable de Rust. Por esta razón debemos instalar luego el toolchain *Nightly* con:

    $ rustup install nightly

Si tu aplicación en Rocket deja de funcionar después de actualizar las dependencias, es muy probable que necesites actualizar también el toolchain:

    $ rustup update

## Generación del nuevo proyecto

En Rust, la herramienta utilizada para gestionar proyectos, dependencias y compilaciones se llama *Cargo*. Cargo es una herramienta que automatiza gran cantidad de tareas y es la que vas a estar utilizando más a menudo cuando estés trabajando con Rust.

Para generar la estructura inicial de nuestra aplicación ejecutamos:

    $ cargo new loadavg-api --bin
    $ cd loadavg-api/

## Instalación de Rocket

Ahora que tenemos la estructura inicial de nuestro proyecto, añadimos a *Rocket* a las dependencias del mismo. Como se mencionó anteriormente, `cargo` es utilizado para gestionar las dependencias, y esto lo hace a través del archivo `Cargo.toml` que se encuentra en la raíz del proyecto.

Dentro del archivo `Cargo.toml`, usamos la sección `[dependencies]` para definir qué *crates* utilizará nuestro proyecto. Por defecto, estos *crates* son descargados desde el repositorio central comunitario en [crates.io](https://crates.io/). Así, añadimos `rocket` y `rocket_codegen`, que incluye herramientas de generación automática de código que nos va a ahorrar una gran cantidad de trabajo al implementar nuestra API.

    [dependencies]
    rocket = "0.1.5"
    rocket_codegen = "0.1.5"

La próxima vez que se ejecute `cargo build` o `cargo run`, él mismo se encargará de encontrar, descargar y construir las dependencias del proyecto.

## Implementación del API

Ya con todo en sitio, podemos comenzar a implementar nuestra API.

### Modelado inicial

Como paso inicial, vamos a modelar datos que nuestra aplicación manejará. Teniendo una fuerte base en la programación funcional, Rust hace uso de *tipos de datos* para este fin.

#### Tipos de datos

Rust permite definir datos tipados con características arbitrarias a través de `struct`s. De manera que, si queremos tener una abstracción de la carga promedio del sistema, o `Load Average`, podríamos modelarlo de la siguiente manera:

    struct LoadAvg {
        last: f32,  // last minute load average
        last5: f32,  // last 5 minutes load average
        last15: f32  // last 15 minutes load average
    }

Estamos creando una estructura `LoadAvg` con 3 *campos*, cada uno de los cuales tiene tipo `f32`, que maneja números flotantes de 32 bits. Esta estructura es en si un nuevo tipo de datos que abstrae la carga promedio del sistema. Si observamos la especificación de la respuesta que esperan nuestros clientes, podemos darnos cuenta de que el tipo de datos `LoadAvg` es muy similar.

Añadimos este nuevo tipo de datos al código de nuestra aplicación en `src/main.rs`, y continuamos.

#### Comportamiento de los datos

Las `struct`s en Rust no son, necesariamente, estructuras estáticas. Al contrario, estas permiten modelar el comportamiento de un dato a través de *métodos*, muy al estilo de las clases en los lenguajes de programación orientados a objetos. Para añadir métodos a un tipo de datos, utilizamos la palabra clave `impl`.

Si queremos implementar un constructor para nuestro tipo `LoadAvg`, podemos hacerlo de la siguiente manera:

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


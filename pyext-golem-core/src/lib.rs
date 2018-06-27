#![allow(non_upper_case_globals)]
#![allow(unused_variables)]

extern crate actix;
extern crate futures;
#[macro_use]
extern crate cpython;
extern crate net;
extern crate pyo3;

#[macro_use]
pub mod python;
pub mod core;
pub mod error;
pub mod event;
pub mod logging;
pub mod network;

use std::sync::{Arc, Mutex};
use cpython::*;

use core::*;
use error::*;
use python::*;

static mut CORE: Core = Core {
    network: None,
    rx: None,
};

py_exception!(golem_core, CoreError);
py_class!(class CoreNetwork |py| {
    data queue: PyShared;

    def __new__(_cls, queue: PyObject) -> PyResult<CoreNetwork> {
        let queue = Arc::new(Mutex::new(Some(queue)));
        CoreNetwork::create_instance(py, queue)
    }

    def running(&self) -> PyResult<bool> {
        unsafe {
            Ok(CORE.running())
        }
    }

    def run(
        &self,
        host: PyString,
        port: PyInt
    ) -> PyResult<bool> {
        unsafe {
            if CORE.running() {
                return Ok(false);
            }

            match CORE.run(host, port) {
                Ok(_) => {
                    let queue = self.queue(py).clone();
                    match CORE.run_rx_queue(queue) {
                        true => Ok(true),
                        false => {
                            let err = ModuleError::new(
                                ErrorKind::Other,
                                "rx channel end error",
                                None
                            );
                            Err(err.into())
                        },
                    }
                },
                Err(e) => Err(e.into())
            }
        }
    }

    def connect(
        &self,
        protocol: PyLong,
        host: PyString,
        port: PyLong
    ) -> PyResult<bool> {
        unsafe {
            if CORE.running() {
                return Ok(false);
            }

            match CORE.connect(protocol, host, port) {
                Ok(_) => Ok(true),
                Err(e) => Err(e.into()),
            }
        }
    }

    def disconnect(
        &self,
        protocol: PyLong,
        host: PyString,
        port: PyLong
    ) -> PyResult<bool> {
        unsafe {
            if CORE.running() {
                return Ok(false);
            }

            match CORE.disconnect(protocol, host, port) {
                Ok(_) => Ok(true),
                Err(e) => Err(e.into()),
            }
        }
    }

    def send(
        &self,
        protocol: PyLong,
        host: PyString,
        port: PyLong,
        protocol_id: PyLong,
        message: Vec<u8>
    ) -> PyResult<bool> {
        unsafe {
            if CORE.running() {
                return Ok(false);
            }

            match CORE.send(protocol, host, port, protocol_id, message) {
                Ok(_) => Ok(true),
                Err(e) => Err(e.into()),
            }
        }
    }
});

py_module_initializer!(libgolem_core, initlibgolem_core, PyInit_libgolem_core, |py, m| {
    try!(m.add(py, "__doc__", "Rust implementation of golem-core."));
    try!(m.add(py, "CoreNetwork", py.get_type::<CoreNetwork>()));
    try!(m.add(py, "CoreError", py.get_type::<CoreError>()));
    Ok(())
});

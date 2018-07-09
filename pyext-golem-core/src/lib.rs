#![allow(non_upper_case_globals)]
#![allow(unused_variables)]

extern crate actix;
extern crate futures;
#[macro_use]
extern crate cpython;
extern crate net;
extern crate pyo3;
extern crate spin;

#[macro_use]
pub mod python;
pub mod core;
pub mod error;
pub mod logging;

use std::time::Duration;
use cpython::*;

use core::*;
use error::ModuleError;
use python::event_into;

static mut CORE: Core = Core{
    network: None,
    rx: None,
};


py_exception!(libgolem_core, CoreError);
py_class!(class CoreNetwork |py| {

    def __new__(_cls) -> PyResult<CoreNetwork> {
        CoreNetwork::create_instance(py)
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
                return Err(ModuleError::not_running().into());
            }

            match CORE.run(py, host, port) {
                Ok(_) => Ok(true),
                Err(e) => Err(e.into())
            }
        }
    }

    def stop(&self) -> PyResult<bool> {
        unsafe {
            if !CORE.running() {
                return Err(ModuleError::not_running().into());
            }

            match CORE.stop() {
                Ok(_) => Ok(true),
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
            if !CORE.running() {
                return Err(ModuleError::not_running().into());
            }

            match CORE.connect(py, protocol, host, port) {
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
            if !CORE.running() {
                return Err(ModuleError::not_running().into());
            }

            match CORE.disconnect(py, protocol, host, port) {
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
        message: PyBytes
    ) -> PyResult<bool> {
        unsafe {
            if !CORE.running() {
                return Ok(false);
            }

            match CORE.send(py, protocol, host, port, protocol_id, message) {
                Ok(_) => Ok(true),
                Err(e) => Err(e.into()),
            }
        }
    }

    def poll(&self, timeout: PyLong) -> PyResult<Option<PyTuple>> {
        unsafe {
            if !CORE.running() {
                return Ok(None);
            }

            match CORE.rx {
                None => Ok(None),
                Some(ref rx) => {
                    let timeout: i64 = timeout.into_object().extract(py)?;
                    if timeout > 0 {
                        // convert from s to ms
                        let duration = Duration::from_millis((timeout * 1000) as u64);
                        // give control back to Python's VM for the time
                        match py.allow_threads(|| rx.lock().recv_timeout(duration)) {
                            Ok(ev) => Ok(Some(event_into(py, ev))),
                            Err(e) => Ok(None),
                        }
                    } else {
                        // in-place poll
                        match rx.lock().recv() {
                            Ok(ev) => Ok(Some(event_into(py, ev))),
                            Err(e) => Ok(None),
                        }
                    }
                }
            }
        }
    }
});

py_module_initializer!(
    libgolem_core,
    initlibgolem_core,
    PyInit_libgolem_core,
    |py, m| {
        try!(m.add(py, "__doc__", "Rust implementation of golem-core."));
        try!(m.add(py, "CoreNetwork", py.get_type::<CoreNetwork>()));
        try!(m.add(py, "CoreError", py.get_type::<CoreError>()));
        Ok(())
    }
);

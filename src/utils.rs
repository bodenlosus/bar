use std::future::Future;
use std::time;

pub fn thread_context() -> glib::MainContext {
    glib::MainContext::thread_default().unwrap_or_else(glib::MainContext::default)
}

pub fn spawn<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    thread_context().spawn_local(future);
}

pub fn set_interval<T: FnMut() -> glib::ControlFlow + 'static>(
    func: T,
    time: u32,
) -> glib::SourceId {
    let mut delta = time::Instant::now();
    let mut func = func;
    glib::idle_add_local(move || {
        if delta.elapsed().as_millis() < time as u128 {
            return glib::ControlFlow::Continue;
        }
        delta = time::Instant::now();
        func()
    })
}

macro_rules! unwrap_or_return {

    ($expression:expr, Result) => {
        match $expression {
            Ok(value) => value,
            Err(err) => {
                println!("Error: {}", err);
                return;
            }
        }      
    };
    ($expression:expr, Option) => {
        match $expression {
            Some(value) => value,
            None => {
                return;
            }
        }      
    };
}

pub(crate) use unwrap_or_return;
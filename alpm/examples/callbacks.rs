use std::{cell::RefCell, rc::Rc};

use alpm::{Alpm, Event, SigLevel};

fn main() {
    // initialise the handle
    let handle = Alpm::new("/", "tests/db").unwrap();

    // set the logcb to log messages, we don't need to store state so pass in a unit struct ()
    handle.set_log_cb((), |loglevel, msg, _data| print!("{:?} {}", loglevel, msg));

    // set the logcb to log messages and how many messages there have been so far.
    // This makes use of the state argument
    handle.set_log_cb(0, |loglevel, msg, data| {
        print!("{} {:?} {}", data, loglevel, msg);
        *data += 1;
    });

    // set the logcb to log messages and how many messages there have been so far.
    // This makes use of the state argument
    // Wrap in an Rc RefCell to allow it to also be read outside the callback
    let number = Rc::new(RefCell::new(0));
    handle.set_log_cb(number.clone(), |loglevel, msg, data| {
        let data: &RefCell<i32> = &*data;
        let mut number = data.borrow_mut();
        print!("{} {:?} {}", number, loglevel, msg);
        *number += 1;
    });

    // use the event callback to print events.
    handle.set_event_cb((), |event, _data| match event.event() {
        Event::TransactionStart => println!("transaction start"),
        Event::TransactionDone => println!("transaction done"),
        _ => (),
    });

    // register any databases you wish to use
    handle
        .register_syncdb("core", SigLevel::USE_DEFAULT)
        .unwrap();
    handle
        .register_syncdb("extra", SigLevel::USE_DEFAULT)
        .unwrap();
    handle
        .register_syncdb("community", SigLevel::USE_DEFAULT)
        .unwrap();

    println!("final value of number was: {}", number.borrow())
}

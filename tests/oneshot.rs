extern crate futures;

use std::sync::mpsc;
use std::thread;

use futures::{Future, Poll};
use futures::future::{lazy, finished};
use futures::sync::oneshot::*;

mod support;
use support::*;

#[test]
fn smoke_poll() {
    let (mut tx, rx) = channel::<u32>();
    let mut task = futures::executor::spawn(lazy(|| {
        assert!(tx.poll_cancel().unwrap().is_not_ready());
        assert!(tx.poll_cancel().unwrap().is_not_ready());
        drop(rx);
        assert!(tx.poll_cancel().unwrap().is_ready());
        assert!(tx.poll_cancel().unwrap().is_ready());
        finished::<(), ()>(())
    }));
    assert!(task.poll_future(unpark_noop()).unwrap().is_ready());
}

#[test]
fn cancel_notifies() {
    let (tx, rx) = channel::<u32>();
    let (tx2, rx2) = mpsc::channel();

    WaitForCancel { tx: tx }.then(move |v| tx2.send(v)).forget();
    drop(rx);
    rx2.recv().unwrap().unwrap();
}

struct WaitForCancel {
    tx: Sender<u32>,
}

impl Future for WaitForCancel {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<(), ()> {
        self.tx.poll_cancel()
    }
}

#[test]
fn cancel_lots() {
    let (tx, rx) = mpsc::channel::<(Sender<_>, mpsc::Sender<_>)>();
    let t = thread::spawn(move || {
        for (tx, tx2) in rx {
            WaitForCancel { tx: tx }.then(move |v| tx2.send(v)).forget();
        }

    });

    for _ in 0..20000 {
        let (otx, orx) = channel::<u32>();
        let (tx2, rx2) = mpsc::channel();
        tx.send((otx, tx2)).unwrap();
        drop(orx);
        rx2.recv().unwrap().unwrap();
    }
    drop(tx);

    t.join().unwrap();
}

/****************************************************************************
*
*   queue/mod.rs
*   ioq
*
*   Copyright 2015 Tyler Cole
*
***/

#[cfg(windows)]
mod windows;

#[cfg(windows)]
pub use self::windows::*;


/****************************************************************************
*
*   Context
*
***/

pub trait Context {
    fn to_event (self: Box<Self>, bytes: u32) -> Event;
}


/****************************************************************************
*
*   Context
*
***/

pub trait Custom {
    fn execute (&mut self);
}


/****************************************************************************
*
*   Event
*
***/

pub enum Event {
    Custom(Box<Custom>),
    Dummy, // temporary to prevent compiler errors
}


/****************************************************************************
*
*   Tests
*
***/

#[cfg(test)]
mod tests {
    use super::*;

    const NUMBER: u32 = 1234;

    struct TestEvent {
        n: u32,
    }

    impl Custom for TestEvent {
        fn execute (&mut self) {
            assert_eq!(NUMBER, self.n);
        }
    }

    //=======================================================================
    #[test]
    fn create_queue () {
        assert!(Queue::new().is_ok());
    }

    //=======================================================================
    #[test]
    fn custom_event () {
        let queue = Queue::new();
        assert!(queue.is_ok());
        let queue = queue.unwrap();

        let event = Box::new(TestEvent { n: NUMBER });
        assert!(queue.enqueue(event).is_ok());

        let event = queue.dequeue();
        assert!(event.is_ok());
        if let Event::Custom(mut event) = event.unwrap() {
            event.execute();
        }
        else {
            panic!();
        }
    }
}
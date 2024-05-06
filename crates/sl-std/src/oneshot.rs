//! Provides a single-producer, single consumer channel that can only carry a single message

use std::{
    cell::UnsafeCell,
    fmt, mem,
    ptr::NonNull,
    sync::atomic::{fence, AtomicU8, Ordering},
    thread::{self, Thread},
};

const INITIAL: u8 = 0b011;
const WAITING_FOR_RECEIVER: u8 = 0b100;
const WAITING_FOR_SENDER: u8 = 0b000;
const WAKING: u8 = 0b001;
const DONE: u8 = 0b010;

pub struct Channel<T> {
    state: AtomicU8,
    message: UnsafeCell<mem::MaybeUninit<T>>,
    waker: UnsafeCell<mem::MaybeUninit<Thread>>,
}

impl<T> Channel<T> {
    #[must_use]
    pub fn create() -> (Sender<T>, Receiver<T>) {
        let channel = NonNull::from(Box::leak(Box::default()));

        let sender = Sender { channel };

        let receiver = Receiver { channel };

        (sender, receiver)
    }
}

pub struct Sender<T> {
    /// The [Channel] that a message will be written to
    ///
    /// # Ownership
    /// Before sending a message, this [Sender] owns the [Channel].
    /// After that, the ownership is (implicitly) transferred to the [Receiver].
    channel: NonNull<Channel<T>>,
}

unsafe impl<T> Send for Sender<T> {}

pub struct Receiver<T> {
    channel: NonNull<Channel<T>>,
}
unsafe impl<T> Send for Receiver<T> {}

#[derive(Clone, Copy)]
pub struct ReceiveError;

/// Contains the message so it can be recovered
pub struct SendError<T>(T);

impl<T> Sender<T> {
    pub fn send(self, message: T) -> Result<(), SendError<T>> {
        let channel_ptr = self.channel;

        // We do all cleanup in here
        mem::forget(self);

        let channel = unsafe { channel_ptr.as_ref() };

        // SAFETY: There can only be one message sent (since this method consumes the sender)
        //         Therefore, there is no message already in the channel
        unsafe { channel.set_message(message) };

        // At this point we can be in the INITIAL or WAITING_FOR_SENDER
        // states. Adding 1 turns them into WAITING_FOR_RECEIVER / UNPARKING
        // states respectively.
        // TODO: Can we relax this ordering?
        let previous_state = channel.state.fetch_add(1, Ordering::SeqCst);

        match previous_state {
            INITIAL => {
                // Our job is done
                Ok(())
            },
            WAITING_FOR_SENDER => {
                // The waker must not be taken before this
                fence(Ordering::Acquire);

                // SAFETY: There is a receiver waiting on the channel, otherwise we wouldn't be in this state
                let waker = unsafe { channel.take_waker() };

                channel.state.swap(WAITING_FOR_RECEIVER, Ordering::AcqRel);

                waker.unpark();

                Ok(())
            },
            DONE => {
                // Receiver already disconnected

                // SAFETY: We know theres a valid message because we just set it
                let message = unsafe { channel.take_message() };

                // SAFETY: Since the Receiver is gone it is our responsibility to drop the channel
                unsafe { drop(Box::from_raw(channel_ptr.as_ptr())) }

                Err(SendError(message))
            },
            _ => unreachable!(),
        }
    }
}

impl<T> Default for Channel<T> {
    fn default() -> Self {
        Self {
            state: AtomicU8::new(INITIAL),
            message: UnsafeCell::new(mem::MaybeUninit::uninit()),
            waker: UnsafeCell::new(mem::MaybeUninit::uninit()),
        }
    }
}

impl<T> Channel<T> {
    unsafe fn set_message(&self, message: T) {
        (*self.message.get()).write(message);
    }

    unsafe fn set_waker(&self, waker: Thread) {
        (*self.waker.get()).write(waker);
    }

    #[must_use]
    unsafe fn take_waker(&self) -> Thread {
        self.waker.get().read().assume_init()
    }

    #[must_use]
    unsafe fn take_message(&self) -> T {
        self.message.get().read().assume_init()
    }

    unsafe fn drop_waker(&self) {
        let _ = self.take_waker();
    }

    unsafe fn drop_message(&self) {
        let _ = self.take_message();
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        let channel = unsafe { self.channel.as_ref() };

        let previous_state = channel.state.fetch_xor(0b001, Ordering::Relaxed);
        match previous_state {
            INITIAL => {
                // Great, nothing to do
            },
            WAITING_FOR_SENDER => {
                // There's a receiver waiting, wake it up so it can proceed.

                // Make our change visible to the receiver
                fence(Ordering::Acquire);

                // SAFETY: There is a receiver waiting, so there's also a
                //         corresponding waker
                let waker = unsafe { channel.take_waker() };

                channel.state.store(DONE, Ordering::Release);

                waker.unpark();
            },
            DONE => {
                // Receiver disconnected, just drop the channel
                unsafe { self.channel.drop_in_place() }
            },
            _ => unreachable!(),
        };
    }
}

impl<T> Receiver<T> {
    pub fn receive_blocking(self) -> Result<T, ReceiveError> {
        // SAFETY: The Sender is never going to deallocate the channel while
        //         we're alive.
        let channel_ptr = self.channel;
        let channel = unsafe { self.channel.as_ref() };

        // We clean up the resources in this call
        mem::forget(self);

        match channel.state.load(Ordering::Acquire) {
            INITIAL => {
                // There is no message yet, we should go to sleep.
                // SAFETY: There is no waker yet, because we're (evidently) not asleep.
                unsafe {
                    channel.set_waker(thread::current());
                }

                match channel.state.swap(WAITING_FOR_SENDER, Ordering::Release) {
                    INITIAL => {
                        loop {
                            thread::park();

                            match channel.state.load(Ordering::Acquire) {
                                WAITING_FOR_RECEIVER => {
                                    // Got a message while we were sleeping

                                    // SAFETY: We're only in this state if the Sender set a message for us
                                    let message = unsafe { channel.take_message() };

                                    // SAFETY: Since the Sender sent its message it is now our job to drop the channel
                                    unsafe {
                                        drop(Box::from_raw(channel_ptr.as_ptr()));
                                    }

                                    return Ok(message);
                                },
                                DONE => {
                                    // Sender was dropped while we were sleeping
                                    // SAFETY: Since we're still alive its our responsibility
                                    //         to free the channel
                                    unsafe {
                                        drop(Box::from_raw(channel_ptr.as_ptr()));
                                    }

                                    return Err(ReceiveError);
                                },
                                WAKING | WAITING_FOR_SENDER => {
                                    // This is a spurious wakeup, there's nothing to do yet
                                },
                                _ => unreachable!(),
                            }
                        }
                    },
                    WAITING_FOR_RECEIVER => {
                        // Got a message while we were setting the waker
                        fence(Ordering::Acquire);

                        // SAFETY: Since the Sender switched from the INITIAL to the WAITING_FOR_RECEIVER state
                        //         it did not use the waker that we just set (so we need to drop it)
                        unsafe {
                            channel.drop_waker();
                        }

                        let message = unsafe { channel.take_message() };

                        // SAFETY: Since the Sender sent its message it is now our job to drop the channel
                        unsafe {
                            drop(Box::from_raw(channel_ptr.as_ptr()));
                        }

                        Ok(message)
                    },
                    DONE => {
                        // The sender disconnected while we were parking

                        // SAFETY: The sender didn't touch the waker that we set earlier
                        unsafe {
                            channel.drop_waker();
                        }

                        // SAFETY: Since the sender won't free the channel while we are alive it is
                        //         our responsibility to drop it
                        unsafe {
                            drop(Box::from_raw(channel_ptr.as_ptr()));
                        }

                        Err(ReceiveError)
                    },
                    _ => unreachable!(),
                }
            },
            WAITING_FOR_RECEIVER => {
                // There already is a message, we don't need to block at all!
                let message = unsafe { channel.take_message() };

                // SAFETY: Since the Sender sent its message it is now our job to drop the channel
                unsafe {
                    drop(Box::from_raw(channel_ptr.as_ptr()));
                }

                Ok(message)
            },
            DONE => {
                // Sender disconnected (without sending a message)

                // SAFETY: Since the sender won't free the channel while we are alive it is
                //         our responsibility to drop it
                unsafe {
                    drop(Box::from_raw(channel_ptr.as_ptr()));
                }

                Err(ReceiveError)
            },
            _ => unreachable!(),
        }
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        // SAFETY: We're still alive (at this point), so the Sender didn't drop the channel yet
        let channel = unsafe { self.channel.as_ref() };

        match channel.state.swap(DONE, Ordering::Acquire) {
            INITIAL => {
                // Nothing to do
            },
            WAITING_FOR_RECEIVER => {
                // There is a message that we didn't read

                // SAFETY: There is a message
                unsafe {
                    channel.drop_message();
                }

                // SAFETY: After the sender sends a message its up to us to drop the channel
                unsafe { drop(Box::from_raw(self.channel.as_ptr())) }
            },
            DONE => {
                // Sender was dropped

                // SAFETY: Since we were not dropped when the sender was dropped, it is our
                //         responsibility to drop the channel
                unsafe { drop(Box::from_raw(self.channel.as_ptr())) }
            },
            _ => unreachable!(),
        }
    }
}

impl<T> fmt::Debug for SendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "Receiver disconnected".fmt(f)
    }
}

impl fmt::Debug for ReceiveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "Sender disconnected".fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use super::*;

    #[test]
    fn send_u8() {
        let (sender, receiver) = Channel::create();

        sender.send(42u8).unwrap();
        let msg = receiver.receive_blocking().unwrap();

        assert_eq!(msg, 42);
    }

    #[test]
    fn drop_receiver() {
        let (sender, receiver) = Channel::create();
        drop(receiver);

        assert_matches!(sender.send(42_u8), Err(SendError(42)));
    }

    #[test]
    fn drop_sender() {
        let (sender, receiver) = Channel::<u8>::create();

        drop(sender);
        assert!(receiver.receive_blocking().is_err());
    }

    #[test]
    fn drop_sender_then_receiver() {
        let (sender, receiver) = Channel::<u8>::create();

        drop(sender);
        drop(receiver);
    }

    #[test]
    fn drop_receiver_then_sender() {
        let (sender, receiver) = Channel::<u8>::create();

        drop(receiver);
        drop(sender);
    }
}

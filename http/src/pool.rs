use std::future::Future;
use std::marker::PhantomData;
use std::thread::{spawn, JoinHandle};

use anyhow::Result;
use crossbeam_channel::{bounded, Receiver, Sender};
use tokio::runtime::Builder;

pub struct Thread<C> {
    handle: JoinHandle<()>,
    _phantom: PhantomData<fn() -> C>,
}

impl<C> Thread<C>
where
    C: Send + 'static,
{
    pub fn spawn<F, Fut>(rx: Receiver<C>, f: F) -> Self
    where
        F: Fn(C) -> Fut + Send + 'static,
        Fut: Future<Output = ()>,
    {
        Self {
            handle: spawn(move || {
                Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap()
                    .block_on(async move {
                        while let Ok(c) = rx.recv() {
                            f(c).await
                        }
                    })
            }),
            _phantom: PhantomData,
        }
    }

    #[allow(unused)]
    pub fn join(self) -> std::thread::Result<()> {
        self.handle.join()
    }
}

pub struct Pool<C> {
    threads: Vec<Thread<C>>,
    channel: (Sender<C>, Receiver<C>),
}

impl<C> Pool<C>
where
    C: Send + Sync + 'static,
{
    pub fn new(capacity: usize) -> Self {
        Self {
            threads: Vec::new(),
            channel: bounded(capacity),
        }
    }

    pub fn spawn<F, Fut>(&mut self, f: F)
    where
        F: Fn(C) -> Fut + Send + 'static,
        Fut: Future<Output = ()>,
    {
        let (_, rx) = &self.channel;
        self.threads.push(Thread::spawn(rx.clone(), f));
    }

    pub fn spawn_many<F, Fut>(&mut self, count: usize, f: F)
    where
        F: Fn(C) -> Fut + Clone + Send + 'static,
        Fut: Future<Output = ()>,
    {
        [0..count].into_iter().for_each(|_| {
            self.spawn(f.clone());
        })
    }

    pub fn send(&self, connection: C) -> Result<()> {
        let (tx, _) = &self.channel;
        tx.try_send(connection)?;
        Ok(())
    }
}

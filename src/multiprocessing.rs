use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::mpsc;

pub struct ThreadPool {
    pool: threadpool::ThreadPool,
}

impl ThreadPool {
    pub fn new() -> Self {
        let worker_count = num_cpus::get();
        ThreadPool::with_workers(worker_count)
    }

    pub fn with_workers(count: usize) -> Self {
        assert!(count > 0, "worker count cannot be {}", count);

        ThreadPool {
            pool: threadpool::ThreadPool::new(count),
        }
    }

    pub fn worker_count(&self) -> usize {
        self.pool.max_count()
    }

    pub fn imap<F, I, T, R>(&self, f: F, inputs: I) -> IMapIterator<R>
        where F: Fn(T) -> R + Send + Sync + 'static,
              I: IntoIterator<Item = T>,
              T: Send + 'static,
              R: Send + 'static,
    {
        let f = Arc::new(f);
        let (tx, rx) = mpsc::channel();
        let mut total = 0;
        for (i, input) in inputs.into_iter().enumerate() {
            total += 1;
            let f = f.clone();
            let tx = tx.clone();
            self.pool.execute(move || {
                let result = f(input);
                if let Err(_) = tx.send((i, result)) {
                    // ignore error
                }
            });
        }
        IMapIterator::new(rx, total)
    }
}

pub struct IMapIterator<T> {
    rx: mpsc::Receiver<(usize, T)>,
    results: BTreeMap<usize, T>,
    next: usize,
    total: usize,
}

impl<T> IMapIterator<T> {
    fn new(rx: mpsc::Receiver<(usize, T)>, total: usize) -> Self {
        IMapIterator {
            rx: rx,
            results: BTreeMap::new(),
            next: 0,
            total: total,
        }
    }
}

impl<T> Iterator for IMapIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        while self.next < self.total {
            if let Some(result) = self.results.remove(&self.next) {
                self.next += 1;
                return Some(result);
            }

            let (i, result) = match self.rx.recv() {
                Ok((i, result)) => (i, result),
                Err(_) => {
                    self.next = self.total;
                    break;
                },
            };
            assert!(i >= self.next, "got {}, next is {}", i, self.next);
            assert!(!self.results.contains_key(&i), "{} already exists", i);
            self.results.insert(i, result);
        }
        None
    }
}

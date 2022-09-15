use super::mapper::Mapper;
use std::collections::VecDeque;
use std::thread;

pub struct Pipeline<I, M>
where
    I: Iterator,
    <I as Iterator>::Item: Send + 'static,
    M: Mapper<I::Item> + Clone + Send + 'static,
    <M as Mapper<I::Item>>::Out: Send + 'static,
{
    input: I,
    queue: VecDeque<crossbeam_channel::Receiver<M::Out>>,
    dispatch: crossbeam_channel::Sender<(I::Item, crossbeam_channel::Sender<M::Out>)>,
    workers: Vec<thread::JoinHandle<()>>,
}

impl<I, M> Pipeline<I, M>
where
    I: Iterator,
    <I as Iterator>::Item: Send + 'static,
    M: Mapper<I::Item> + Clone + Send + 'static,
    <M as Mapper<I::Item>>::Out: Send + 'static,
{
    pub fn new(n_workers: usize, m: M, input: I) -> Pipeline<I, M> {
        let n_workers = n_workers.min(1);
        let (dispatch, dispatch_rx): (
            crossbeam_channel::Sender<(I::Item, crossbeam_channel::Sender<M::Out>)>,
            _,
        ) = crossbeam_channel::bounded(0);
        let mut workers = Vec::with_capacity(n_workers);

        for _ in 0..n_workers {
            let mut m = m.clone();
            let dispatch_rx = dispatch_rx.clone();
            let handle = thread::spawn(move || loop {
                match dispatch_rx.recv() {
                    Ok((in_val, respond)) => {
                        let out_val = m.apply(in_val);
                        respond.send(out_val).unwrap();
                    }
                    Err(_) => break,
                }
            });
            workers.push(handle)
        }

        Pipeline {
            input,
            dispatch,
            workers,
            queue: VecDeque::with_capacity(n_workers),
        }
    }
}

impl<I, M> Drop for Pipeline<I, M>
where
    I: Iterator,
    <I as Iterator>::Item: Send + 'static,
    M: Mapper<I::Item> + Clone + Send + 'static,
    <M as Mapper<I::Item>>::Out: Send + 'static,
{
    fn drop(&mut self) {
        let (dummy, _) = crossbeam_channel::bounded(1);
        self.dispatch = dummy;
        for worker in self.workers.drain(..) {
            worker.join().unwrap();
        }
    }
}

impl<I, M> Iterator for Pipeline<I, M>
where
    I: Iterator,
    <I as Iterator>::Item: Send + 'static,
    M: Mapper<I::Item> + Clone + Send + 'static,
    <M as Mapper<I::Item>>::Out: Send + 'static,
{
    type Item = <M as Mapper<I::Item>>::Out;

    fn next(&mut self) -> Option<Self::Item> {
        while self.queue.len() < self.workers.len() {
            match self.input.next() {
                Some(v) => {
                    let (tx, rx) = crossbeam_channel::bounded(1);
                    self.queue.push_back(rx);
                    self.dispatch.send((v, tx)).unwrap();
                }
                None => break,
            }
        }

        match self.queue.pop_front() {
            Some(rx) => Some(rx.recv().unwrap()),
            None => None,
        }
    }
}

pub trait PipelineMap<I, M>
where
    I: Iterator,
    <I as Iterator>::Item: Send + 'static,
    M: Mapper<I::Item> + Clone + Send + 'static,
    <M as Mapper<I::Item>>::Out: Send + 'static,
{
    fn plmap(self, n_workers: usize, m: M) -> Pipeline<I, M>;
}

impl<I, M> PipelineMap<I, M> for I
where
    I: Iterator,
    <I as Iterator>::Item: Send + 'static,
    M: Mapper<I::Item> + Clone + Send + 'static,
    <M as Mapper<I::Item>>::Out: Send + 'static,
{
    fn plmap(self, n_workers: usize, m: M) -> Pipeline<I, M> {
        Pipeline::new(n_workers, m, self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_pipeline() {
        for (i, v) in (0..100).plmap(5, |x| x * 2).enumerate() {
            let i = i as i32;
            assert_eq!(i * 2, v)
        }
        assert_eq!((0..100).plmap(2, |x| x * 2).count(), 100);
    }
}

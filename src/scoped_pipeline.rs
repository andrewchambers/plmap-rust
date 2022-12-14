use {super::mapper::Mapper, std::collections::VecDeque};

/// ScopedPipeline is a wrapper around a worker pool and implements
/// iterator. Usually they should be created via the PipelineMap
/// extension trait and calling plmap on an iterator.
///
/// ScopedPipeline differs from Pipeline in that it uses a std::thread::Scope
/// and allows non 'static lifetimes.
pub struct ScopedPipeline<'scope, 'env, I, M>
where
    I: Iterator,
    I::Item: Send + 'env,
    M: Mapper<I::Item> + Clone + Send + 'env,
    M::Out: Send + 'env,
{
    mapper: M,
    input: I,
    queue: VecDeque<crossbeam_channel::Receiver<M::Out>>,
    dispatch: crossbeam_channel::Sender<(I::Item, crossbeam_channel::Sender<M::Out>)>,
    _worker_scope: &'scope crossbeam_utils::thread::Scope<'env>,
    workers: Vec<crossbeam_utils::thread::ScopedJoinHandle<'scope, ()>>,
}

impl<'scope, 'env, I, M> ScopedPipeline<'scope, 'env, I, M>
where
    I: Iterator,
    I::Item: Send + 'env,
    M: Mapper<I::Item> + Clone + Send + 'env,
    M::Out: Send + 'env,
{
    pub fn new(
        worker_scope: &'scope crossbeam_utils::thread::Scope<'env>,
        n_workers: usize,
        mapper: M,
        input: I,
    ) -> ScopedPipeline<'scope, 'env, I, M> {
        let (dispatch, dispatch_rx): (
            crossbeam_channel::Sender<(_, crossbeam_channel::Sender<M::Out>)>,
            _,
        ) = crossbeam_channel::bounded(0);
        let mut workers = Vec::with_capacity(n_workers);

        for _ in 0..n_workers {
            let mut mapper = mapper.clone();
            let dispatch_rx = dispatch_rx.clone();
            let handle = worker_scope.spawn(move |_| {
                while let Ok((in_val, respond)) = dispatch_rx.recv() {
                    let out_val = mapper.apply(in_val);
                    respond.send(out_val).unwrap();
                }
            });
            workers.push(handle)
        }

        ScopedPipeline {
            mapper,
            input,
            dispatch,
            workers,
            _worker_scope: worker_scope,
            queue: VecDeque::with_capacity(n_workers + 1),
        }
    }
}

impl<'scope, 'env, I, M> Drop for ScopedPipeline<'scope, 'env, I, M>
where
    I: Iterator,
    I::Item: Send + 'env,
    M: Mapper<I::Item> + Clone + Send + 'env,
    M::Out: Send + 'env,
{
    fn drop(&mut self) {
        let (dummy, _) = crossbeam_channel::bounded(1);
        self.dispatch = dummy;
        for worker in self.workers.drain(..) {
            worker.join().unwrap();
        }
    }
}

impl<'scope, 'env, I, M> Iterator for ScopedPipeline<'scope, 'env, I, M>
where
    I: Iterator,
    I::Item: Send + 'env,
    M: Mapper<I::Item> + Clone + Send + 'env,
    M::Out: Send + 'env,
{
    type Item = <M as Mapper<I::Item>>::Out;

    fn next(&mut self) -> Option<Self::Item> {
        if self.workers.is_empty() {
            return self.input.next().map(|v| self.mapper.apply(v));
        }

        while self.queue.len() < self.workers.len() + 1 {
            match self.input.next() {
                Some(v) => {
                    let (tx, rx) = crossbeam_channel::bounded(1);
                    self.queue.push_back(rx);
                    self.dispatch.send((v, tx)).unwrap();
                }
                None => break,
            }
        }

        self.queue.pop_front().map(|rx| rx.recv().unwrap())
    }
}

/// ScopedPipelineMap can be imported to add the scoped_plmap function to iterators.
pub trait ScopedPipelineMap<'scope, 'env, I, M>
where
    I: Iterator,
    I::Item: Send + 'env,
    M: Mapper<I::Item> + Clone + Send + 'env,
    M::Out: Send + 'env,
{
    fn scoped_plmap(
        self,
        worker_scope: &'scope crossbeam_utils::thread::Scope<'env>,
        n_workers: usize,
        m: M,
    ) -> ScopedPipeline<'scope, 'env, I, M>;
}

impl<'scope, 'env, I, M> ScopedPipelineMap<'scope, 'env, I, M> for I
where
    I: Iterator,
    I::Item: Send + 'env,
    M: Mapper<I::Item> + Clone + Send + 'env,
    M::Out: Send + 'env,
{
    fn scoped_plmap(
        self,
        worker_scope: &'scope crossbeam_utils::thread::Scope<'env>,
        n_workers: usize,
        m: M,
    ) -> ScopedPipeline<'scope, 'env, I, M> {
        ScopedPipeline::new(worker_scope, n_workers, m, self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scoped_parallel_pipeline() {
        crossbeam_utils::thread::scope(|s| {
            for w in 0..3 {
                for (i, v) in (0..100).scoped_plmap(s, w, |x| x * 2).enumerate() {
                    let i = i as i32;
                    assert_eq!(i * 2, v)
                }
                assert_eq!((0..100).scoped_plmap(s, w, |x| x * 2).count(), 100);
            }
        })
        .unwrap()
    }
}

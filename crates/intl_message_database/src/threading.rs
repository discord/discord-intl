use std::sync::mpsc::channel;
use threadpool::ThreadPool;

/// Returns a reasonable number of threads to utilize for processing on the
/// running system. This examines whether logical cpus and physical cpus are
/// different and decides to use a _majority_ of available resources, but
/// without taking over the whole system. That's important because this library
/// is almost always run in the context of other work being done (i.e., as a
/// Node addon, as part of a bundler process, etc.), and that shouldn't have the
/// world stopped because of this (albeit fast) process consuming everything.
///
/// This can be overridden using the `INTL_CONCURRENCY` environment variable for
/// situations where it's expected that the computed count will be wrong (e.g.,
/// in Docker environments that incorrectly report system resources).
pub(crate) fn get_reasonable_thread_count() -> usize {
    if let Ok(concurrency) = std::env::var("INTL_CONCURRENCY") {
        if let Ok(requested_count) = concurrency.parse::<usize>() {
            return requested_count;
        }
    }

    let physical = num_cpus::get_physical();
    let logical = num_cpus::get();
    // Use half of the cores on small machines
    if logical < 8 {
        return logical / 2;
    }
    // If hyperthreading is enabled on medium machines, use the physical count.
    if logical > physical && physical <= 12 {
        return physical;
    }

    // Otherwise use 2/3 of available resources.
    logical * 2 / 3
}

/// For each element of `data`, run `thread_func` in a separate thread using a thread pool with a
/// pre-determined size (i.e., some threads may be reused if there are more items than threads
/// available). The result for each element is sent back to the main thread, where `processor` is
/// called with it as the argument.
pub(crate) fn run_in_thread_pool<
    Data: IntoIterator<Item = T> + ExactSizeIterator,
    T: Send + Sync + 'static, // Data being processed
    V: Send + 'static,        // Value returned from thread_func
    R,                        // Return value of the processor
    P: Fn(T) -> V + Copy + Send + Sync + 'static,
    F: FnMut(V) -> R,
>(
    data: Data,
    thread_func: P,
    mut processor: F,
) -> anyhow::Result<Vec<R>> {
    let num_jobs = data.len();
    let pool = ThreadPool::new(get_reasonable_thread_count());
    let (tx, rx) = channel();
    for datum in data {
        let tx = tx.clone();

        pool.execute(move || {
            let result = thread_func(datum);
            tx.send(result)
                .expect("Failed to send processing result from thread pool back to supervisor");
        });
    }

    let mut results = Vec::with_capacity(num_jobs);
    for result in rx.iter().take(num_jobs) {
        results.push(processor(result));
    }
    Ok(results)
}

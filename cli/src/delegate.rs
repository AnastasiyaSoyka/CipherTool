use std::sync::mpsc::Sender;

use rayon::iter::{IntoParallelIterator, ParallelIterator};

/**
 * Create a serial buffer and send it to the [sender].
 */
pub fn create_serial(sender: Sender<Vec<u8>>, closure: impl FnOnce() -> Vec<u8>) {
    let buffer = closure();

    sender.send(buffer).unwrap();
}

/**
 * Create a parallel buffer and send it to the [sender].
 */
pub fn create_parallel(sender: Sender<Vec<u8>>, count: Option<usize>, closure: impl Fn() -> Vec<u8> + Send + Sync) {
    let max = count.unwrap_or(1);
    let range = 0..max;

    range.into_par_iter().for_each(|_| {
        let buffer = closure();

        sender.send(buffer).unwrap();
    });
}

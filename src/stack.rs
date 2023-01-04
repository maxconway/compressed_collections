use serde::Deserialize;
use serde::Serialize;

use crate::compression::compress;
use crate::compression::decompress;
use crate::ChunkSize;

/// A stack which automatically compresses itself over a certain size
///
/// # Examples
///
/// ```
/// use compressed_collections::Stack;
///
/// let mut compressed_stack = Stack::new();
/// for _ in 0..(1024) {
///     compressed_stack.push(1.0);
/// }
/// ```
///
/// # Panics
///
/// This function should not panic (except on out of memory conditions). If it does, please submit an issue.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct Stack<T> {
    uncompressed_buffer: Vec<T>,
    compressed_storage: Vec<Vec<u8>>,
    chunk_size: usize,
    compression_level: i32,
}

impl<T> Stack<T> {
    /// Constructor with default options
    pub fn new() -> Stack<T> {
        Stack::new_with_options(ChunkSize::Default, 0)
    }

    /// Constructor with customisable options
    ///
    /// - `chunksize` size of chunks to compress, see [`ChunkSize`]
    /// - `compression_level` Brotli compression level (0-9) default is 0
    ///
    /// # Low stability
    /// This constructor is dependent on the internal implementation, so it is likely to change more frequently than [`Stack::new`]
    pub fn new_with_options(chunksize: ChunkSize, compression_level: i32) -> Stack<T> {
        let elementsize = std::mem::size_of::<T>();
        let chunk_size = match chunksize {
            ChunkSize::SizeElements(x) => x,
            ChunkSize::SizeBytes(x) => x / elementsize,
            ChunkSize::SizeMB(x) => x * 1024 * 1024 / elementsize,
            ChunkSize::Default => 10 * 1024 * 1024 / elementsize,
        };
        let uncompressed_buffer = Vec::new();
        let compressed_storage = Vec::new();
        Stack {
            uncompressed_buffer,
            compressed_storage,
            chunk_size,
            compression_level,
        }
    }
    /// Push an item onto the stack
    pub fn push(&mut self, value: T)
    where
        T: Serialize,
    {
        self.uncompressed_buffer.push(value);
        if self.uncompressed_buffer.len() >= self.chunk_size {
            let compressed = compress(&self.uncompressed_buffer, self.compression_level);
            self.compressed_storage.push(compressed);
            self.uncompressed_buffer.clear();
        }
    }
    /// Pop an item off the stack
    pub fn pop(&mut self) -> Option<T>
    where
        T: for<'a> Deserialize<'a>,
    {
        if self.uncompressed_buffer.is_empty() {
            if let Some(x) = self.compressed_storage.pop() {
                self.uncompressed_buffer = decompress(&x);
            }
        }
        self.uncompressed_buffer.pop()
    }
}

impl<T> Default for Stack<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Iterator for Stack<T>
where
    T: Serialize + for<'a> Deserialize<'a>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item>
    where
        T: Serialize + for<'a> Deserialize<'a>,
    {
        self.pop()
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn simple_test() {
        let mut big_vec = Vec::new();
        let mut compressed_stack = Stack::new_with_options(ChunkSize::SizeElements(1024 * 9), 0);
        for _ in 0..(1024 * 10) {
            big_vec.push(1.0);
            compressed_stack.push(1.0);
        }
        loop {
            let a = big_vec.pop();
            let b = compressed_stack.pop();
            assert!(a == b);
            if a.is_none() | b.is_none() {
                break;
            }
        }
    }

    #[test]
    fn iter_test() {
        let mut big_vec = Vec::new();
        let mut compressed_stack = Stack::new_with_options(ChunkSize::SizeElements(1024 * 9), 0);
        for _ in 0..(1024 * 10) {
            big_vec.push(1.0);
            compressed_stack.push(1.0);
        }
        let mut big_vec_it = big_vec.into_iter();
        let mut compressed_stack_it = compressed_stack.into_iter();
        loop {
            let a = big_vec_it.next();
            let b = compressed_stack_it.next();
            assert!(a == b);
            if a.is_none() | b.is_none() {
                break;
            }
        }
    }
}

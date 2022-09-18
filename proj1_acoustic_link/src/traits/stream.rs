/// Read continuously data of type [`T`] from [`InStream`].
/// Might encounter error of type [`E`]
pub trait InStream<T, E> {
    /// Read data to a slice.
    /// The function call should return instantly,
    /// do not wait for incoming data.
    fn read(&mut self, buf: &mut [T]) -> Result<usize, E>;
    /// Read data to fill a slice.
    /// The function will not return until the slice is filled.
    fn read_exact(&mut self, buf: &mut [T]) -> Result<(), E>;
}
/// Write data of type [`T`] continuously into [`OutStream`].
/// Might encounter error of type [`E`]
pub trait OutStream<T, E> {
    /// Write data from a
    /// The function call should return instantly,
    /// do not wait for data to be pushed or buffer to be flushed.
    fn write(&mut self, buf: &[T]) -> Result<usize, E>;
    /// Write data from a slice.
    /// The function will not return until all the data are written.
    fn write_exact(&mut self, buf: &[T]) -> Result<(), E>;
}

/// An [`IoStream`] is capable of reading and writing.
pub trait IoStream<T, E>: InStream<T, E> + OutStream<T, E> {}
impl<T, E, S> IoStream<T, E> for S where S: InStream<T, E> + OutStream<T, E> {}

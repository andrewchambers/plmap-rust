/// Mapper is a type that can map values from In to Out,
/// You can implement this trait to plmap on types other than closures.
///
/// Mapper is essentially to FnMut(In) -> Out,
/// but users can implement it without experimental
/// features.
pub trait Mapper<In> {
    /// The output type.
    type Out;
    /// Run the mapping function converting In to Out.
    fn apply(&mut self, v: In) -> Self::Out;
}

impl<A, B, F> Mapper<A> for F
where
    F: FnMut(A) -> B,
{
    type Out = B;

    fn apply(&mut self, x: A) -> Self::Out {
        self(x)
    }
}

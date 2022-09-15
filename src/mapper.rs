pub trait Mapper<InType> {
    type Out;
    fn apply(&mut self, v: InType) -> Self::Out;
}

impl<A, B, F> Mapper<A> for F
where
    F: Fn(A) -> B,
{
    type Out = B;

    fn apply(&mut self, x: A) -> Self::Out {
        self(x)
    }
}

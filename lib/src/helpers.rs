pub fn replace_slice<T>(source: &mut [T], from: &[T])
    where
        T: Clone + PartialEq,
{
    source[..from.len()].clone_from_slice(from);
}

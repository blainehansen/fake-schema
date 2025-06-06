pub(crate) fn zip_longest<I, J>(left: I, right: J) -> impl Iterator<Item = (Option<I::Item>, Option<J::Item>)>
where
	I: IntoIterator,
	J: IntoIterator,
{
	let mut left = left.into_iter();
	let mut right = right.into_iter();
	std::iter::from_fn(move || match (left.next(), right.next()) {
		(None, None) => None,
		(x, y) => Some((x, y)),
	})
}

#[derive(Debug)]
pub(crate) struct Emp;

impl std::fmt::Display for Emp {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "")
	}
}

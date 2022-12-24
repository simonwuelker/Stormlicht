pub trait Backend {
	fn init(width: usize, height: usize) -> Result<Self, String> where Self: Sized;
}
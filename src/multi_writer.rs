//! The module that implements a writer for writing to multiple destinations
//! simultaneously.

// Uses
use std::io::{Error, Result, Write};

/// A writer for writing to multiple destinations simultaneously.
pub struct MultiWriter<'a> {
	children: Vec<&'a mut dyn Write>,
}

impl<'a> MultiWriter<'a> {
	pub fn new(children: Vec<&'a mut dyn Write>) -> Self {
		Self { children }
	}
}

impl Write for MultiWriter<'_> {
	fn write(&mut self, buf: &[u8]) -> Result<usize> {
		if self.children.is_empty() {
			return Ok(buf.len());
		}

		let written_byte_counts = self
			.children
			.iter_mut()
			.map(|w| w.write(buf))
			.collect::<Result<Vec<_>>>()?;

		let first_byte_count = written_byte_counts[0];
		if !written_byte_counts
			.iter()
			.all(|byte_count| *byte_count == first_byte_count)
		{
			return Err(Error::other(
				"underlying writers wrote different byte counts",
			));
		}

		Ok(first_byte_count)
	}

	fn flush(&mut self) -> Result<()> {
		self.children.iter_mut().try_for_each(Write::flush)
	}
}

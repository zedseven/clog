// Uses
use std::num::ParseIntError;

pub fn vec_to_arr<T, const N: usize>(vec: Vec<T>) -> [T; N] {
	vec.try_into()
		.unwrap_or_else(|v: Vec<T>| panic!("expected a Vec of length {} but it was {}", N, v.len()))
}

// Sourced from: https://stackoverflow.com/a/52992629
pub fn parse_hex_str(s: &str) -> Result<Vec<u8>, ParseIntError> {
	(0..s.len())
		.step_by(2)
		.map(|i| u8::from_str_radix(&s[i..i + 2], 16))
		.collect()
}

pub fn bytes_to_str(bytes: &[u8]) -> String {
	fn nibble_to_char(num: u8) -> char {
		(match num {
			0..=9 => num + 0x30,
			10..=15 => (num - 10) + 0x61,
			_ => unreachable!("there should be nothing higher than 0xf"),
		}) as char
	}

	let mut result = String::with_capacity(bytes.len() * 2);
	for &byte in bytes {
		result.push(nibble_to_char((0b1111_0000 & byte) >> 4));
		result.push(nibble_to_char(0b0000_1111 & byte));
	}
	result
}

// Uses
use anyhow::{anyhow, Result};

pub fn vec_to_arr<T, const N: usize>(vec: Vec<T>) -> [T; N] {
	vec.try_into()
		.unwrap_or_else(|v: Vec<T>| panic!("expected a Vec of length {} but it was {}", N, v.len()))
}

pub fn parse_hex_str(hex_asm: &str) -> Vec<u8> {
	let mut hex_bytes = hex_asm
		.as_bytes()
		.iter()
		.filter_map(|b| match b {
			b'0'..=b'9' => Some(b - b'0'),
			b'a'..=b'f' => Some(b - b'a' + 10),
			b'A'..=b'F' => Some(b - b'A' + 10),
			_ => None,
		})
		.fuse();

	let mut bytes = Vec::new();
	while let (Some(h), Some(l)) = (hex_bytes.next(), hex_bytes.next()) {
		bytes.push(h << 4 | l);
	}
	bytes
}

pub fn parse_hex_str_strict(hex_asm: &str) -> Result<Vec<u8>> {
	if !hex_asm.is_ascii()
		|| hex_asm.contains(|c| !matches!(c as u8, b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F'))
	{
		Err(anyhow!("input hex string is invalid"))
	} else {
		Ok(parse_hex_str(hex_asm))
	}
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

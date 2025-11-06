//! The module for clipboard operations.

// Uses
use std::{
	env::consts::OS,
	io::{stdin, Read},
};

use anyhow::{anyhow, Result};
use copypasta::{ClipboardContext, ClipboardProvider};

use crate::constants::APPLICATION_PROPER_NAME;

// Constants
/// A list of operating systems where the clipboard is known to not be
/// persistent once the application exits.
const NON_PERSISTENT_CLIPBOARD_OSES: &[&str] =
	&["linux", "openbsd", "freebsd", "netbsd", "solaris"];

/// Copies a string to the OS clipboard.
pub fn copy_str_to_clipboard(contents: &str) -> Result<()> {
	let mut context =
		ClipboardContext::new().map_err(|_| anyhow!("unable to create a clipboard context"))?;

	context
		.set_contents(contents.to_owned())
		.map_err(|_| anyhow!("unable to set clipboard contents"))?;

	// https://github.com/alacritty/copypasta/issues/49
	eprintln!();
	if NON_PERSISTENT_CLIPBOARD_OSES.contains(&OS) {
		eprintln!(
			"Note: On this OS, the clipboard contents will be lost when the application that set \
			 them exits."
		);
		eprintln!(
			"To avoid this, {APPLICATION_PROPER_NAME} will wait until Enter is pressed before \
			 exiting so that the contents can be pasted where they're needed."
		);
		await_user_input();
	} else {
		eprintln!("Note: The output has been copied to the clipboard!");
	}

	Ok(())
}

fn await_user_input() {
	// Throw away the result because it does not matter in this case
	stdin().read_exact(&mut [0]).ok();
}

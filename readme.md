# synchronous-autocomplete-rs

**Simple, synchronous autocompletion.** I'm a Rust beginner, so the code might be unelegant. 🙈

[![crates.io version](https://img.shields.io/crates/v/synchronous_autocomplete.svg)](https://crates.io/crates/synchronous_autocomplete)
[![build status](https://api.travis-ci.org/derhuerst/synchronous-autocomplete-rs.svg?branch=master)](https://travis-ci.org/derhuerst/synchronous-autocomplete-rs)
![ISC-licensed](https://img.shields.io/github/license/derhuerst/synchronous-autocomplete-rs.svg)
[![chat on gitter](https://badges.gitter.im/derhuerst.svg)](https://gitter.im/derhuerst)
[![support me on Patreon](https://img.shields.io/badge/support%20me-on%20patreon-fa7664.svg)](https://patreon.com/derhuerst)


## Installing

Put this into your `Cargo.toml`:

```toml
synchronous_autocomplete = "0.1.0"
```


## Usage

Check the [full guide over at `synchronous-autocomplete`](https://github.com/derhuerst/synchronous-autocomplete/blob/0b02a4ab52ccb5ce4ad50b274711f571bb65ae9d/readme.md#usage), the JS equivalent.

```rust
extern crate synchronous_autocomplete;
use synchronous_autocomplete::{Item, build_index, run};

fn main() {
	let items = vec![
		Item {
			id: String::from("apple"),
			name: String::from("Juicy sour Apple."),
			weight: 3.0
		},
		Item {
			id: String::from("banana"),
			name: String::from("Sweet juicy Banana!"),
			weight: 2.0
		},
		Item {
			id: String::from("pomegranate"),
			name: String::from("Sour Pomegranate"),
			weight: 5.0
		}
	];

	let idx = build_index(items);

	println!("normal {:?}", run(&idx, String::from("sour"), false, false));
	println!("completion {:?}", run(&idx, String::from("bana"), true, false));
	println!("fuzzy {:?}", run(&idx, String::from("aplle"), false, true));
}
```


## Contributing

If you have a question or have difficulties using synchronous-autocomplete-rs, please double-check your code and setup first. If you think you have found a bug or want to propose a feature, refer to [the issues page](https://github.com/derhuerst/synchronous-autocomplete-rs/issues).

extern crate synchronous_autocomplete;
extern crate easybench;
use synchronous_autocomplete::{Index, run};
use std::collections::HashMap;
use easybench::bench;

// todo: DRY
fn sample_index() -> Index {
	let mut tokens = HashMap::new();
	tokens.insert(String::from("one"), vec![0]);
	tokens.insert(String::from("two"), vec![1]);
	tokens.insert(String::from("three"), vec![1]);
	tokens.insert(String::from("four"), vec![0, 1]);

	let mut scores = HashMap::new();
	scores.insert(String::from("one"), 1.0 / 2.0);
	scores.insert(String::from("two"), 1.0 / 2.0);
	scores.insert(String::from("three"), 1.0 / 2.0);
	scores.insert(String::from("four"), 1.0);

	Index {
		tokens,
		scores,
		weights: vec![10.0, 20.0],
		nr_of_tokens: vec![2, 3],
		original_ids: vec![String::from("A"), String::from("B")]
	}
}

fn main() {
	let idx = sample_index();
	println!("completion {}", bench(|| run(&idx, String::from("fou"), true, false)));
	println!("fuzzy {}", bench(|| run(&idx, String::from("there"), false, true)));
}
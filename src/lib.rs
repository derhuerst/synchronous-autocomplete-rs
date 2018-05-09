#[macro_use] extern crate slugify;
use slugify::slugify;
use std::collections::HashMap;

pub struct Index {
	pub tokens: HashMap<String, Vec<i32>>,
	pub scores: HashMap<String, f64>,
	pub weights: Vec<f64>,
	pub nr_of_tokens: Vec<i32>,
	pub original_ids: Vec<String>
}

pub struct Item {
	pub id: String,
	pub name: String,
	pub weight: f64
}

pub fn build_index(items: Vec<Item>) -> Index {
	let mut index = Index {
		tokens: HashMap::new(),
		scores: HashMap::new(),
		weights: vec![],
		nr_of_tokens: vec![],
		original_ids: vec![]
	};

	let mut id = 0;
	for item in &items {
		let tokens = slugify!(&item.name, separator = " ");
		let tokens = tokens.split_whitespace();

		let mut nr_of_tokens = 0;
		for token in tokens {
			let mut ids = index.tokens
				.entry(token.to_string())
				.or_insert(vec![]);
			ids.push(id); // todo: check if added
			nr_of_tokens += 1;
		}
		index.weights.push(item.weight);
		index.nr_of_tokens.push(nr_of_tokens);
		index.original_ids.push(item.id.clone());
		id += 1;
	}

	for (token, ids) in index.tokens.iter() {
		let score: f64 = ids.len() as f64 / items.len() as f64;
		index.scores.insert(token.to_string(), score);
	}

	index
}
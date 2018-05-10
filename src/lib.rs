#[macro_use] extern crate slugify;
extern crate levenshtein;
extern crate float_ord;
use slugify::slugify;
use levenshtein::levenshtein;
use float_ord::FloatOrd;
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

pub struct Result {
	pub id: String,
	pub weight: f64,
	pub relevance: f64,
	pub score: f64
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

	println!("{:?}", index);
	index
}

fn by_fragment(idx: &Index, fragment: String, completion: bool, fuzzy: bool) -> HashMap<i32, f64> {
	let mut results = HashMap::new();
	let l = fragment.len();

	if idx.tokens.contains_key(&fragment) {
		let relevance = 1.0
			+ idx.scores.get(&fragment).expect("invalid index")
			+ (l as f64).sqrt();

		let ids = idx.tokens.get(&fragment).expect("invalid index");
		for &id in ids.iter() {
			let mut total_relevance = results.entry(id).or_insert(0.0);
			*total_relevance += relevance;
		}
	}

	if completion || fuzzy {
		for token in idx.tokens.keys() {
			if *token == fragment { continue; }
			let mut relevance;

			let token_length = token.len();
			let token_slice: String = token.chars().take(l).collect();
			if completion && token_length > l && fragment == token_slice {
				relevance = 1.0 // add-one smoothing
					+ idx.scores.get(token).expect("invalid index")
					+ l as f64 / token_length as f64;
			} else if fuzzy {
				let distance = levenshtein(&fragment, token);
				if distance > 3 { continue; }
				relevance = ( // add-one smoothing
					(1.0 + idx.scores.get(token).expect("invalid index"))
					/ (1.0 + distance as f64)
				);
			} else { continue; }

			let ids = idx.tokens.get(token).expect("invalid index");
			for &id in ids.iter() {
				let mut total_relevance = results.entry(id).or_insert(0.0);
				*total_relevance += relevance;
			}
		}
	}

	results
}

pub fn run(idx: &Index, query: String, completion: bool, fuzzy: bool) -> Vec<Result> {
	if query == "" {
		return vec![];
	}

	let mut results = HashMap::new();
	let fragments = slugify!(&query, separator = " ");
	let fragments = fragments.split_whitespace();

	for fragment in fragments {
		let ids = with_fragment(idx, fragment.to_string(), completion, fuzzy);
		for (id, relevance) in ids {
			// todo: exclude items that don't match all fragments
			let nr_of_tokens = idx.nr_of_tokens.get(id as usize).expect("invalid index");
			let mut total_relevance = results
				.entry(id)
				.or_insert(1.0 / *nr_of_tokens as f64);
			*total_relevance *= relevance;
		}
	}

	// todo: nr of results param
	let mut scores: Vec<f64> = Vec::with_capacity(3);
	let mut items: Vec<Result> = Vec::with_capacity(3);
	for (id, relevance) in results {
		let weight = idx.weights.get(id as usize).expect("invalid index");
		let score = relevance * weight;

		// todo: most likely, this can be done more elegantly
		let score_f = FloatOrd(score);
		let pos = scores
			.binary_search_by(|s| score_f.cmp(&FloatOrd(*s)))
			.unwrap_or_else(|e| e);
		if pos >= 3 { continue; } // score too low
		// todo: most likely, this can be done faster than insert & truncate
		scores.insert(pos, score);
		scores.truncate(3);

		let original_id = idx.original_ids.get(id as usize).expect("invalid index");
		items.insert(pos, Result {
			id: original_id.to_string(),
			weight: *weight,
			relevance,
			score
		});
		items.truncate(3);
	}

	items
}
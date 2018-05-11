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

#[cfg(test)]
mod test {
	use super::*;

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

	#[test]
	fn build_index_works() {
		let items = vec![
			Item {
				id: String::from("A"),
				name: String::from("One fOUr!"),
				weight: 10.0
			},
			Item {
				id: String::from("B"),
				name: String::from("two THREE four?"),
				weight: 20.0
			}
		];
		let exp = sample_index();
		let act = build_index(items);

		// deep_equal(act.tokens, exp.tokens)
		assert_eq!(act.tokens.len(), exp.tokens.len());
		for (token, exp_ids) in exp.tokens {
			let act_ids = act.tokens.get(&token).expect("");
			for i in 0..(exp_ids.len() - 1) {
				assert_eq!(act_ids.get(i), exp_ids.get(i));
			}
		}

		// deep_equal(act.scores, exp.scores)
		assert_eq!(act.scores.len(), exp.scores.len());
		for (token, exp_score) in exp.scores {
			let act_score = act.scores.get(&token).expect("");
			assert_eq!(*act_score, exp_score);
		}

		// deep_equal(act.weights, exp.weights)
		assert_eq!(act.weights.len(), exp.weights.len());
		for i in 0..(exp.weights.len() - 1) {
			let act_weight = act.weights.get(i).expect("");
			let exp_weight = exp.weights.get(i).expect("");
			assert_eq!(act_weight, exp_weight);
		}

		// deep_equal(act.nr_of_tokens, exp.nr_of_tokens)
		assert_eq!(act.nr_of_tokens.len(), exp.nr_of_tokens.len());
		for i in 0..(exp.nr_of_tokens.len() - 1) {
			let act_nr = act.nr_of_tokens.get(i).expect("");
			let exp_nr = exp.nr_of_tokens.get(i).expect("");
			assert_eq!(act_nr, exp_nr);
		}

		// deep_equal(act.original_ids, exp.original_ids)
		assert_eq!(act.original_ids.len(), exp.original_ids.len());
		for i in 0..(exp.original_ids.len() - 1) {
			let act_id = act.original_ids.get(i).expect("");
			let exp_id = exp.original_ids.get(i).expect("");
			assert_eq!(act_id, exp_id);
		}
	}

	#[test]
	fn build_index_handles_duplicate_tokens() {
		let items = vec![
			Item {
				id: String::from("A"),
				name: String::from("foo bar foo"), // "foo" twice
				weight: 10.0
			},
			Item {
				id: String::from("B"),
				name: String::from("foo baz"),
				weight: 20.0
			}
		];
		let idx = build_index(items);

		let foo = idx.tokens.get("foo").expect("missing foo IDs");
		assert_eq!(foo.len(), 3);
		assert_eq!(*foo.get(0).expect("missing foo[0]"), 0);
		assert_eq!(*foo.get(1).expect("missing foo[1]"), 0);
		assert_eq!(*foo.get(2).expect("missing foo[2]"), 1);

		let nrs = idx.nr_of_tokens;
		assert_eq!(nrs.len(), 2);
		assert_eq!(*nrs.get(0).expect("missing idx.nr_of_tokens[0]"), 3);
		assert_eq!(*nrs.get(1).expect("missing idx.nr_of_tokens[1]"), 2);
	}

	#[test]
	fn with_fragment_exact() {
		let idx = sample_index();
		let res = with_fragment(&idx, String::from("four"), false, false);

		assert_eq!(res.len(), 2);
		// 1 + scores[fragment] + sqrt(fragment_length)
		assert_eq!(*res.get(&0).expect(""), 1.0 + 1.0 + 2.0);
		assert_eq!(*res.get(&1).expect(""), 1.0 + 1.0 + 2.0);
	}

	#[test]
	fn with_fragment_completion() {
		let idx = sample_index();
		let res = with_fragment(&idx, String::from("fou"), true, false);

		assert_eq!(res.len(), 2);
		// 1 + scores[fragment] + fragment_length / token_length
		let a = res.get(&0).expect("missing result for A");
		assert_eq!(*a, 1.0 + 1.0 + 3.0 / 4.0);
		let b = res.get(&1).expect("missing result for B");
		assert_eq!(*b, 1.0 + 1.0 + 3.0 / 4.0);
	}

	#[test]
	fn with_fragment_fuzzy() {
		let idx = sample_index();
		let res = with_fragment(&idx, String::from("there"), false, true);

		assert_eq!(res.len(), 1);
		// (1 + scores[fragment]) / (1 + levenshtein_distance)
		let b = res.get(&1).expect("missing result for B");
		assert_eq!(*b, (1.0 + 0.5) / (1.0 + levenshtein("there", "three") as f64));
	}

	#[test]
	fn run_calculates_correctly() {
		let idx = sample_index();
		let res = run(&idx, String::from("fou"), true, false);

		assert_eq!(res.len(), 2);
		let a = res.get(0).expect("missing 1st result");
		let b = res.get(1).expect("missing 2nd result");

		// (1 + scores[token] + fragment_length / token_length) / nr_of_tokens[id]
		let a_rel = (1.0 + 1.0 + 3.0 / 4.0) / 2.0;
		let b_rel = (1.0 + 1.0 + 3.0 / 4.0) / 3.0;
		// relevance * pow(weights[id], 1 / 3)
		let a_score = a_rel * (10.0 as f64).powf(1.0 / 3.0);
		let b_score = b_rel * (20.0 as f64).powf(1.0 / 3.0);

		assert_eq!(a.id, String::from("A"));
		assert_eq!(a.relevance, a_rel);
		assert_eq!(a.score, a_score);

		assert_eq!(b.id, String::from("B"));
		assert_eq!(b.relevance, b_rel);
		assert_eq!(b.score, b_score);
	}

	#[test]
	fn run_handles_duplicate_tokens() {
		// A: "foo bar foo", weight 10
		// B: "foo baz", weight 20
		let mut tokens = HashMap::new();
		tokens.insert(String::from("foo"), vec![0, 0, 1]);
		tokens.insert(String::from("bar"), vec![0]);
		tokens.insert(String::from("baz"), vec![1]);
		let mut scores = HashMap::new();
		scores.insert(String::from("foo"), 2.0 / 2.0);
		scores.insert(String::from("bar"), 1.0 / 2.0);
		scores.insert(String::from("baz"), 1.0 / 2.0);
		let idx = Index {
			tokens,
			scores,
			weights: vec![10.0, 20.0],
			nr_of_tokens: vec![3, 2],
			original_ids: vec![String::from("A"), String::from("B")]
		};

		// For the query "foo ba", B should rank higher, except if the duplicate
		// token "foo" in A is taken into account. We assert that it is.
		let res = run(&idx, String::from("foo ba"), true, false);
		let a = res.get(0).expect("missing 1st result");
		let b = res.get(1).expect("missing 2nd result");
		assert_eq!(a.id, String::from("A"));
		assert_eq!(b.id, String::from("B"));
	}

	// todo: run_limits_nr_of_results
}
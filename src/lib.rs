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
		index.weights.push(item.weight);
		index.original_ids.push(item.id.clone());
		// todo
		id += 1;
	}

	index
}
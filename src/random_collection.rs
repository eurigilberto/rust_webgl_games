use std::cell::Cell;

pub struct RandomF32Collection {
    random_values: Vec<f32>,
    current_index: Cell<usize>,
}

impl RandomF32Collection {
    pub fn new(collection_size: usize) -> Self {
        if collection_size == 0{
            panic!("Cannot create empty random collection");
        }

        let mut random_values = Vec::new();
        for _ in 0..collection_size {
			random_values.push(rand::random());
		}

        Self {
            random_values,
            current_index: Cell::new(0),
        }
    }

	pub fn get_random(&self)->f32{
		let value = self.random_values[self.current_index.get()];
		let mut next_index = self.current_index.get() + 1;
		if next_index >= self.random_values.len(){
			next_index = 0;
		}
        self.current_index.set(next_index);
		value
	}

    pub fn set_index(&self, seed: u32){
        let index = (seed as usize) % self.random_values.len();
        self.current_index.set(index);
    }
}

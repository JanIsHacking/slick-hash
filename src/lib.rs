mod hash_table;

use std::collections::hash_map::{DefaultHasher, Entry};
use hash_table::{Capacity, HashTableBase, HashTableBulk, HashTableRemove, Insertion, Named};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::ops::Range;
use std::u64;
use ahash::AHasher;

pub struct SlickHashMetaData {
    offset: usize,
    gap: usize,
    threshold: usize,
}

pub struct SlickHash<Key, Value> {
    main_table_size: usize,
    block_size: usize,
    number_of_blocks: usize,
    max_slick_size: usize,
    max_offset: usize,
    max_threshold: usize,

    main_table: Vec<(Key, Value)>,
    meta_data: Vec<SlickHashMetaData>,
    backyard: HashMap<Key, Value>,
    no_elements_in_main_table: usize
}

impl<Key, Value> SlickHash<Key, Value>
where
    Key: Clone + Eq + PartialEq + Hash + Default,
    Value: Clone + Default,
{
    fn new(capacity: usize) -> Self {
        // Hyper parameters
        let block_size: usize = 10;
        let max_slick_size = block_size * 2;
        let max_offset = block_size;
        let max_threshold = block_size;

        // Other setup
        let main_table_size = capacity;
        assert_eq!(main_table_size % block_size, 0);
        let number_of_blocks: usize = main_table_size / block_size;
        let main_table: Vec<(Key, Value)> = vec![Default::default(); capacity];
        let mut meta_data: Vec<SlickHashMetaData> = Vec::with_capacity(number_of_blocks);
        for _ in 0..number_of_blocks {
            meta_data.push(SlickHashMetaData {
                offset: 0,
                gap: block_size,
                threshold: 0,
            })
        }

        Self {
            main_table_size,
            block_size,
            number_of_blocks,
            max_slick_size,
            max_offset,
            max_threshold,
            main_table,
            meta_data,
            backyard: HashMap::new(),
            no_elements_in_main_table: 0
        }
    }

    fn block_start(&self, block_index: usize) -> usize {
        assert!(block_index < self.number_of_blocks);
        self.block_size * block_index + self.meta_data[block_index].offset
    }

    fn block_end(&self, block_index: usize) -> usize {
        assert!(block_index < self.number_of_blocks);
        if block_index == self.number_of_blocks - 1 {
            return self.main_table_size - self.meta_data[block_index].gap
        }
        self.block_size * block_index + self.block_size + self.meta_data[block_index+1].offset - self.meta_data[block_index].gap
    }

    fn block_range(&self, block_index: usize) -> Range<usize> {
        let start = self.block_start(block_index);
        let end = self.block_end(block_index);
        return start..end
    }

    fn insert_into_backyard(&mut self, key: Key, value: Value) -> Insertion<Value> {
        match self.backyard.entry(key) {
            Entry::Occupied(occ) => Insertion::Occupied(occ.into_mut()),
            Entry::Vacant(vac) => Insertion::Inserted(vac.insert(value)),
        }
    }

    fn slide_gap_from_left(&mut self, block_index: usize) -> bool {
        let mut sliding_block_index = block_index;
        while self.meta_data[sliding_block_index].gap == 0 {
            if (sliding_block_index == 0) || (self.meta_data[sliding_block_index].offset == 0) {
                return false
            }
            sliding_block_index -= 1;
        }

        // If the block only has a gap of one and is empty, it would be squished :(
        // In this case, sliding gap from left is not possible
        let empty_block_has_gap_one = (self.meta_data[sliding_block_index].gap == 1) && (self.block_start(sliding_block_index) == self.block_end(sliding_block_index));
        if empty_block_has_gap_one {
            return false
        }

        // A gap in block at sliding_block_index has been found

        self.meta_data[sliding_block_index].gap -= 1;
        sliding_block_index += 1;
        while sliding_block_index <= block_index {
            // Extends the block by one space ont the left and fills the free spot at the front with the element at the back
            let start_sliding_block = self.block_start(sliding_block_index);
            let end_sliding_block = self.block_end(sliding_block_index);
            self.main_table[start_sliding_block-1] = self.main_table[end_sliding_block-1].clone();
            self.meta_data[sliding_block_index].offset -= 1;
            sliding_block_index += 1;
        }
        self.meta_data[sliding_block_index-1].gap += 1;
        return true;
    }

    fn slide_gap_from_right(&mut self, block_index: usize) -> bool {
        if block_index == self.number_of_blocks-1 {
            return false;
        }

        let mut sliding_block_index = block_index + 1;
        while self.meta_data[sliding_block_index].gap == 0 {
            if (sliding_block_index == self.number_of_blocks-1) ||
                (self.meta_data[sliding_block_index].offset == self.max_offset) {
                return false;
            }
            sliding_block_index += 1;
        }

        // Enforcing the maximum offset
        if self.meta_data[sliding_block_index].offset == self.max_offset {
            return false
        }

        // If the block only has a gap of one and is empty, it would be squished :(
        // In this case, sliding gap from right is not possible
        let empty_block_has_gap_one = (self.meta_data[sliding_block_index].gap == 1) && (self.block_start(sliding_block_index) == self.block_end(sliding_block_index));
        if empty_block_has_gap_one {
            return false
        }

        // A gap in block at sliding_block_index has been found

        // Unwrapping the first loop execution to reduce the gap of the right-most sliding block
        let start_sliding_block = self.block_start(sliding_block_index);
        let end_sliding_block = self.block_end(sliding_block_index);
        self.main_table[end_sliding_block] = self.main_table[start_sliding_block].clone();

        self.meta_data[sliding_block_index].offset += 1;
        self.meta_data[sliding_block_index].gap -= 1;
        sliding_block_index -= 1;


        while sliding_block_index > block_index {
            // Moves the first element to the end of the block and then removes one space on the left of the block by increasing its offset
            let start_sliding_block = self.block_start(sliding_block_index);
            let end_sliding_block = self.block_end(sliding_block_index);
            // Subtracting 1 from end sliding block because the end now reaches into the next block
            // again because the offset of the successive block has already been updated
            self.main_table[end_sliding_block-1] = self.main_table[start_sliding_block].clone();

            self.meta_data[sliding_block_index].offset += 1;
            sliding_block_index -= 1;
        }
        self.meta_data[sliding_block_index].gap += 1;
        return true;
    }

    fn hash_block_index(&self, key: &Key) -> usize {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = hasher.finish() as f64;

        ((hash / (u64::MAX as f64)) * self.number_of_blocks as f64) as usize
    }

    fn hash_threshold(&self, key: &Key) -> usize {
        let mut hasher = AHasher::default();
        key.hash(&mut hasher);
        let hash = hasher.finish() as f64;
        ((hash / (u64::MAX as f64)) * self.max_threshold as f64) as usize
    }

    fn there_is_no_space(&mut self, block_range: &Range<usize>, block_index: usize) -> bool {
        (block_range.len() >= self.max_slick_size) ||
            !(
                self.meta_data[block_index].gap > 0 ||
                    self.slide_gap_from_left(block_index) ||
                    self.slide_gap_from_right(block_index)
            )
    }
}

impl<Key, Value> HashTableBase<Key, Value> for SlickHash<Key, Value>
where
    Key: Clone + Eq + PartialEq + Hash + Default,
    Value: Clone + Default,
{
    fn with_capacity(capacity: impl Capacity) -> Self {
        SlickHash::new(capacity.capacity())
    }

    fn try_insert(&mut self, key_value_pair: (Key, Value)) -> Insertion<Value> {
        let (key, value) = key_value_pair;
        let block_index = self.hash_block_index(&key);
        let block_start = self.block_start(block_index);
        let block_range = self.block_range(block_index);
        if self.hash_threshold(&key) < self.meta_data[block_index].threshold {
            return self.insert_into_backyard(key, value);
        }

        // Searches for the value in the main table, returns a mutable reference on the value on find
        if block_range.len() > 0 {
            // Finds the index if the key is in the block, else the index stays None
            let block_range_elements_as_mut = &self.main_table[block_range.clone()];
            let mut found_index = None;
            for (index, (iter_key, _)) in block_range_elements_as_mut.iter().enumerate() {
                if *iter_key == key {
                    found_index = Some(index);
                    break
                }
            }

            // Returns a mutable reference on the value if the key is found
            if let Some(some_found_index) = found_index {
                return Insertion::Inserted(&mut self.main_table[some_found_index].1)
            }
        }

        // Bumps elements if there is no space or no space can be made by sliding
        // If the block is too large or there is no empty slot usable in the table
        let there_is_no_space = self.there_is_no_space(&block_range, block_index);
        if there_is_no_space
        {
            let mut min_threshold_hash = self.max_threshold+1;
            // Calculating t prime
            // Find the smallest threshold of all keys present
            for (iter_key, _) in &self.main_table[block_range.clone()] {
                // Find the key with the minimum hash
                let key_threshold = self.hash_threshold(iter_key);
                if key_threshold < min_threshold_hash {
                    min_threshold_hash = key_threshold;
                }
            }

            // Asserting that there has been found a minimum threshold
            assert!(min_threshold_hash < self.max_threshold+1);

            // Check if the threshold of the key to add is the smallest
            if self.hash_threshold(&key) < min_threshold_hash {
                min_threshold_hash = self.hash_threshold(&key);
            }
            let t_prime = min_threshold_hash + 1;

            // Scans the existing elements and bumps them if necessary
            self.meta_data[block_index].threshold = t_prime;
            let mut j = block_start;
            let mut block_end = self.block_end(block_index);
            while j < block_end {
                let (iter_key, iter_value) = &self.main_table[j];
                let key_threshold = self.hash_threshold(iter_key);
                if key_threshold < t_prime {
                    self.insert_into_backyard(iter_key.clone(), iter_value.clone());
                    self.no_elements_in_main_table -= 1;
                    self.main_table[j] = self.main_table[block_end-1].clone();
                    self.meta_data[block_index].gap += 1;
                    block_end = self.block_end(block_index);
                } else {
                    j += 1;
                }
            }
            // Bumps the input key-value pair into the backyard if necessary
            if self.hash_threshold(&key) < t_prime {
                return self.insert_into_backyard(key, value)
            }
        }
        // Inserts the input key-value pair at the end of the block and reduces the block's gap by 1
        let current_block_end = self.block_end(block_index);
        self.main_table[current_block_end] = (key, value);
        self.no_elements_in_main_table += 1;
        self.meta_data[block_index].gap -= 1;

        // Displaying the number of elements in the table at the end, assuming the number of inserted elements is 2,000,000
        if self.no_elements_in_main_table + self.backyard.len() == 2_000_000 {
            println!("Final number of elements in main table: {}", self.no_elements_in_main_table);
            println!("Final number of elements in backyard table: {}", self.backyard.len());
        }

        return Insertion::Inserted(&mut self.main_table[current_block_end].1);
    }

    fn get(&self, key: &Key) -> Option<&Value> {
        let block_index = self.hash_block_index(key);
        if self.hash_threshold(key) < self.meta_data[block_index].threshold {
            return self.backyard.get(key)
        }
        let block_range = self.block_range(block_index);
        let key_value_in_main_table = self.main_table[block_range]
            .into_iter()
            .find(|&key_value_pair| key_value_pair.0 == *key);
        match key_value_in_main_table {
            Some(kvp) => Some(&kvp.1),
            None => None,
        }
    }
}

impl<Key, Value> HashTableBulk<Key, Value> for SlickHash<Key, Value> {
    fn bulk_insert(&mut self, key_value_pairs: &[(Key, Value)]) {
        todo!()
    }
}

impl<Key, Value> HashTableRemove<Key, Value> for SlickHash<Key, Value>
where
    Key: Clone + Eq + PartialEq + Hash + Default,
    Value: Clone + Default,
{
    fn remove_entry(&mut self, key: &Key) -> Option<(Key, Value)> {
        let block_index = self.hash_block_index(key);
        let mut remove_value = None;
        if self.hash_threshold(key) < self.meta_data[block_index].threshold {
            remove_value = self.backyard.remove_entry(key)
        }
        for i in self.block_range(block_index) {
            if *key == self.main_table[i].0 {
                let key_value_pair = self.main_table[i].clone();
                self.main_table[i] = self.main_table[self.block_end(block_index)-1].clone();
                self.meta_data[block_index].gap += 1;
                self.no_elements_in_main_table -= 1;
                remove_value = Some(key_value_pair);
                break
            }
        }
        remove_value
    }
}

impl<Key, Value> Named for SlickHash<Key, Value> {
    fn name() -> String {
        "SlickHash".into()
    }
}

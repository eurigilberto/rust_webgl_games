#[cfg(test)]
mod tests {
    use super::super::{SlotKey, Slotmap};

    #[test]
    fn single_value_can_be_pushed_into_empty_slotmap() {
        let mut u32_slotmap = Slotmap::<u32>::with_capacity(100);
        let new_value_key = u32_slotmap.push(20);
        match new_value_key {
            Some(_) => {
                assert_eq!(
                    u32_slotmap.free_list_len(),
                    99,
                    "Free List does not have the correct lenght"
                );
            }
            _ => {
                panic!("Could not push a value")
            }
        };
    }
    #[test]
    fn multiple_values_can_be_pushed_into_empty_slotmap() {
        let mut u32_slotmap = Slotmap::<u32>::with_capacity(100);
        for i in 0..25 {
            match u32_slotmap.push(i) {
                Some(_) => {}
                None => {
                    panic!("Could not push a value")
                }
            }
        }
        assert_eq!(u32_slotmap.capacity(), 100);
        assert_eq!(u32_slotmap.len(), 25, "Slot map should have 25");
        assert_eq!(
            u32_slotmap.free_list_len(),
            75,
            "Free List does not have the correct lenght"
        );
    }

    #[test]
    fn removing_a_value_from_partially_filled_slotmap_frees_a_slot() {
        let mut u32_slotmap = Slotmap::<u32>::with_capacity(100);
        let mut slot_keys = Vec::<SlotKey>::with_capacity(25);
        for i in 0..25 {
            if let Some(key) = u32_slotmap.push(i) {
                slot_keys.push(key);
            } else {
                panic!("Could not push value")
            }
        }
        match u32_slotmap.remove(slot_keys[0]) {
            Some(value) => {
                assert_eq!(value, 0);
                assert_eq!(
                    u32_slotmap.free_list_len(),
                    76,
                    "Free List does not have the correct lenght"
                );
                match u32_slotmap.get_value(&slot_keys[0]) {
                    Some(_) => {
                        panic!("Removing a value should make the key invalid")
                    }
                    None => {}
                }
            }
            None => {
                panic!("No value was returned on remove for a known valid key")
            }
        };
    }

    #[test]
    fn removing_multiple_values_from_partially_filled_slotmap_frees_multiple_slots() {
        let mut u32_slotmap = Slotmap::<u32>::with_capacity(100);
        let mut slot_keys = Vec::<SlotKey>::with_capacity(25);
        for i in 0..25 {
            if let Some(key) = u32_slotmap.push(i) {
                slot_keys.push(key);
            } else {
                panic!("Could not push value")
            }
        }
        let check_value = |expected_value: u32, slot_result: Option<u32>| match slot_result {
            Some(value) => {
                assert_eq!(
                    value, expected_value,
                    "Resulting value -{}- is not the expected one -{}-",
                    value, expected_value
                );
                println!("Removed element {}", value);
            }
            None => {
                panic!("Expected a value to be returned")
            }
        };

        check_value(0, u32_slotmap.remove(slot_keys[0]));
        check_value(1, u32_slotmap.remove(slot_keys[1]));
        check_value(2, u32_slotmap.remove(slot_keys[2]));
        check_value(4, u32_slotmap.remove(slot_keys[4]));
    }

    #[test]
    fn removing_multiple_values_randomly_from_partially_filled_slotmap_frees_multiple_slots(){
    	let mut u32_slotmap = Slotmap::<u32>::with_capacity(100);
        let mut slot_keys = Vec::<(u32, SlotKey)>::with_capacity(25);
        for i in 0..24 {
            if let Some(key) = u32_slotmap.push(i) {
                slot_keys.push((i, key));
            } else {
                panic!("Could not push value")
            }
        }
        let check_value = |expected_value: u32, slot_result: Option<u32>| match slot_result {
    		Some(value) => {
                assert_eq!(
                    value, expected_value,
                    "Resulting value -{}- is not the expected one -{}-",
                    value, expected_value
                );
            }
            None => {
                panic!("Expected a value to be returned");
            }
        };
    	while slot_keys.len() > 10 {
    		println!("--------------------------");
    		let rand_index = rand::random::<usize>() % slot_keys.len();
    		let key = slot_keys.remove(rand_index);
    		println!("Trying to remove slotkey {:?}", key.1);

    		check_value(key.0, u32_slotmap.remove(key.1));
    	}
    	println!("Free list slice {:?}", u32_slotmap.free_list_slice());
    }
    
    #[test]
    fn removing_multiple_values_can_make_free_buckets_to_merge_on_a_partially_filled_slotmap(){
        let mut u32_slotmap = Slotmap::<u32>::with_capacity(100);
        let mut slot_keys = Vec::<SlotKey>::with_capacity(25);
        for i in 0..24 {
            if let Some(key) = u32_slotmap.push(i) {
                slot_keys.push(key);
            } else {
                panic!("Could not push value")
            }
        }

        let check_value = |expected_value: u32, slot_result: Option<u32>| match slot_result {
            Some(value) => {
                assert_eq!(
                    value, expected_value,
                    "Resulting value -{}- is not the expected one -{}-",
                    value, expected_value
                );
    			println!("Removed element {}", value);
            }
            None => {
                panic!("Expected a value to be returned")
            }
        };
    	check_value(0, u32_slotmap.remove(slot_keys[0]));
        check_value(2, u32_slotmap.remove(slot_keys[2]));
        check_value(1, u32_slotmap.remove(slot_keys[1]));
    	check_value(4, u32_slotmap.remove(slot_keys[4]));
        let slice = u32_slotmap.free_list_slice();
        assert_eq!((slice[0], slice[1], slice[2], slice[3]), (0, 1, 2, 4));
    }

    #[test]
    fn an_iterator_can_be_created_from_a_partially_filled_slotmap(){
        let mut u32_slotmap = Slotmap::<u32>::with_capacity(100);
        let mut slot_keys = Vec::<SlotKey>::with_capacity(25);
        for i in 0..24 {
            if let Some(key) = u32_slotmap.push(i) {
                slot_keys.push(key);
            } else {
                panic!("Could not push value")
            }
        }
        let slice = u32_slotmap.free_list_slice();
        for (index, val) in (24..100).enumerate(){
            assert_eq!(slice[index], val);
        }
        for (index, val) in u32_slotmap.get_iter().enumerate(){
            assert_eq!(index as u32, *val, "The stored value is not the correct one | expected {} - found {} |", index as u32, *val);
        }
    }

    #[test]
    fn a_mutable_iterator_can_be_created_from_a_partially_filled_slotmap(){
        let mut u32_slotmap = Slotmap::<u32>::with_capacity(100);
        let mut slot_keys = Vec::<SlotKey>::with_capacity(25);
        for i in 0..24 {
            if let Some(key) = u32_slotmap.push(i) {
                slot_keys.push(key);
            } else {
                panic!("Could not push value")
            }
        }
        let slice = u32_slotmap.free_list_slice();
        for (index, val) in (24..100).enumerate(){
            assert_eq!(slice[index], val);
        }
        for (index, val) in u32_slotmap.get_iter_mut().enumerate(){
            assert_eq!(index as u32, *val, "The stored value is not the correct one | expected {} - found {} |", index as u32, *val);
            *val += 2;
        }
        for (index, val) in u32_slotmap.get_iter().enumerate(){
            let expected_value = index as u32 + 2;
            assert_eq!(expected_value, *val, "The stored value is not the correct one | expected {} - found {} |", expected_value, *val);
        }
    }

    #[test]
    fn values_cannot_be_pushed_to_slotmaps_at_capacity(){
        let mut u32_slotmap = Slotmap::<u32>::with_capacity(100);
        for i in 0..100 {
            if let Some(_) = u32_slotmap.push(i) {
            } else {
                panic!("Could not push value")
            }
        }
        assert_eq!(u32_slotmap.capacity(), 100, "The capacity is not correct");
        match u32_slotmap.push(20) {
            Some(_) => panic!("No value should be pushed if the slot map is full"),
            None => {}
        }
    }

    /*#[test]
    fn capacity_of_a_full_slotmap_can_be_increased(){
        let mut u32_slotmap = Slotmap::<u32>::with_capacity(100);
        for i in 0..100 {
            if let Some(_) = u32_slotmap.push(i) {
            } else {
                panic!("Could not push value")
            }
        }
        assert_eq!(u32_slotmap.capacity(), 100, "The capacity is not correct");
        assert_eq!(u32_slotmap.len(), 100, "All objects were not pushed");

        match u32_slotmap.reserve_exact(10) {
            Some(extra_capacity) => {
                assert_eq!(extra_capacity, 10, "Extra capacity is not correct");
            },
            None => {
                panic!("No extra capacity was added")
            },
        }
        assert_eq!(u32_slotmap.capacity(), 110, "The capacity is not correct");
        let free_list_slice = u32_slotmap.free_list_slice();
        assert_eq!(free_list_slice[0], (100, 110), "The current head does not have the correct structure")
    }*/

    /*#[test]
    fn when_the_capacity_is_increased_the_resulting_bucket_is_merged_with_the_current_tail_if_possible(){
        let mut u32_slotmap = Slotmap::<u32>::with_capacity(100);
        for i in 0..90 {
            if let Some(_) = u32_slotmap.push(i) {
            } else {
                panic!("Could not push value")
            }
        }

        assert_eq!(u32_slotmap.capacity(), 100, "The capacity is not correct");
        assert_eq!(u32_slotmap.len(), 90, "All objects were not pushed");

        match u32_slotmap.reserve_exact(10) {
            Some(_) => {
                panic!("Capacity should not have been increased")
            },
            None => {},
        }

        match u32_slotmap.reserve_exact(20) {
            Some(extra_capacity) => {
                assert_eq!(10, extra_capacity, "The increased capacity is not correct")
            },
            None => {
                panic!("Capacity should have increased")
            },
        }

        let free_list_slice = u32_slotmap.free_list_slice();
        assert_eq!(free_list_slice[0], (90, 110), "Extra capacity bucket was not merged with the tail bucket")
    }*/

    /*#[test]
    fn new_capacity_bucket_is_not_merged_if_the_tail_bucket_is_not_connected_to_it(){
        let mut u32_slotmap = Slotmap::<u32>::with_capacity(100);
        let mut slot_keys = Vec::<SlotKey>::with_capacity(100);
        for i in 0..100 {
            if let Some(key) = u32_slotmap.push(i) {
                slot_keys.push(key);
            } else {
                panic!("Could not push value")
            }
        }

        assert_eq!(u32_slotmap.capacity(), 100, "The capacity is not correct");
        assert_eq!(u32_slotmap.len(), 100, "The length is not correct");
        assert_eq!(u32_slotmap.free_list_slice().len(), 0, "There should not be free slots");

        for i in 0 as usize .. 20{
            u32_slotmap.remove(slot_keys[i]);
        }
        let free_list_slice = u32_slotmap.free_list_slice();
        assert_eq!(free_list_slice.len(), 1, "There should be a single free bucket");
        assert_eq!(free_list_slice[0], (0, 20), "The free bucket should contain all the removed elements");

        match u32_slotmap.reserve_exact(40) {
            Some(extra_capacity) => {
                assert_eq!(extra_capacity, 20, "There should be exactly 20 extra slots");
            },
            None => {
                panic!("Extra space should have been alocated")
            },
        }

        let free_list_slice = u32_slotmap.free_list_slice();
        assert_eq!(free_list_slice.len(), 2, "There should be a single free bucket");

        let expected_slice = [(0,20), (100, 120)];
        for (index, expected) in expected_slice.into_iter().enumerate() {
            assert_eq!(free_list_slice[index], expected, "Free list does not have the correct structure");
        }
    }*/

    #[test]
    /// Alternate delete exmaple slot array [use, free, use, free, use, free, use ...]
    fn stress_testing_with_alternate_deletes_32000_entries(){
        {
            let mut u32_slotmap = Slotmap::<u32>::with_capacity(32000);
            let mut slot_keys = Vec::<SlotKey>::with_capacity(32000);
            for i in 0..32000 {
                if let Some(key) = u32_slotmap.push(i) {
                    slot_keys.push(key);
                } else {
                    panic!("Could not push value")
                }
            }

            for (index, key) in slot_keys.iter().enumerate(){
                if index % 2 == 0{
                    u32_slotmap.remove(*key).unwrap();
                }
            }
        }
    }
}

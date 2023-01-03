#[macro_export]
macro_rules! create_custom_key {
    (
        $struct_name:ident
    ) => {
        use std::ops::Deref;
        #[derive(Clone, Copy)]
        pub struct $struct_name(pub SlotKey);
        impl Deref for $struct_name {
            type Target = SlotKey;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };
}

pub mod prelude;
mod test;
use std::slice::{Iter, IterMut};

#[derive(Debug, Clone, Copy)]
enum ValueOrFreeIndex {
    Value(usize),
    Free(usize),
    End,
}

#[derive(Debug, Clone, Copy)]
pub struct SlotIndex(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Generation(u32);

pub struct Slot {
    index: ValueOrFreeIndex,
    generation: Generation,
}
#[derive(Debug, Clone, Copy)]
pub struct SlotKey {
    index: SlotIndex,
    generation: Generation,
}

pub struct SlotCollection {
    slots: Vec<Slot>,
    free_head: Option<usize>,
}

impl SlotCollection {
    pub fn new(capacity: usize) -> Self {
        let mut slots: Vec<Slot> = Vec::with_capacity(capacity);
        for index in 0..capacity {
            if index < capacity - 1 {
                slots.push(Slot {
                    index: ValueOrFreeIndex::Free(index + 1),
                    generation: Generation(0),
                })
            } else {
                slots.push(Slot {
                    index: ValueOrFreeIndex::End,
                    generation: Generation(0),
                })
            }
        }
        Self {
            slots,
            free_head: Some(0),
        }
    }

    #[inline]
    pub fn is_valid(&self, slot_key: &SlotKey) -> bool {
        let slot = &self.slots[slot_key.index.0];
        if let ValueOrFreeIndex::Value(_) = slot.index {
            return slot.generation == slot_key.generation;
        } else {
            return false;
        }
    }

    pub fn get_value_index(&self, slot_key: &SlotKey) -> Option<usize> {
        if !self.is_valid(slot_key) {
            return None;
        }
        match self.slots[slot_key.index.0].index {
            ValueOrFreeIndex::Value(index) => Some(index),
            _ => None,
        }
    }

    /// Should only be called if the caller checked that the collection is not empty
    pub fn take_slot(&mut self, value_index: usize) -> Option<SlotKey> {
        let free_head_index = self.free_head.unwrap();
        let slot = &self.slots[free_head_index];

        let value_or_free_index = slot.index;
        let generation = slot.generation;

        match value_or_free_index {
            ValueOrFreeIndex::Value(_) => {
                panic!("The free head index points to a filled slot");
            }
            ValueOrFreeIndex::Free(_) | ValueOrFreeIndex::End => {
                if let ValueOrFreeIndex::Free(next_free) = value_or_free_index {
                    self.free_head = Some(next_free);
                } else {
                    self.free_head = None;
                }

                self.slots[free_head_index].index = ValueOrFreeIndex::Value(value_index);
                Some(SlotKey {
                    index: SlotIndex(free_head_index),
                    generation: generation,
                })
            }
        }
    }

    pub fn return_slot(
        &mut self,
        slot_key: SlotKey,
        slot_value_change: Option<(SlotIndex, usize)>,
    ) {
        //Update the value of a slot if a swap happened
        match slot_value_change {
            Some((slot_index, value_index)) => match self.slots[slot_index.0].index {
                ValueOrFreeIndex::Value(_) => {
                    self.slots[slot_index.0].index = ValueOrFreeIndex::Value(value_index);
                }
                ValueOrFreeIndex::Free(_) | ValueOrFreeIndex::End => {
                    panic!("The slot index should be pointing to a taken slot")
                }
            },
            None => {}
        }

        // if the free list is empty, only push the new free slot
        if self.free_head.is_none() {
            let slot = &mut self.slots[slot_key.index.0];
            slot.generation = Generation(slot.generation.0 + 1);
            slot.index = ValueOrFreeIndex::End;
            self.free_head = Some(slot_key.index.0);
            //slot.
        } else {
            let free_head_index = self.free_head.unwrap();
            // if new index is smaller then it has to be the new head
            if free_head_index > slot_key.index.0 {
                self.free_head = Some(slot_key.index.0);
                if let ValueOrFreeIndex::Value(_) = self.slots[slot_key.index.0].index {
                    let slot = &mut self.slots[slot_key.index.0];
                    slot.generation.0 += 1;
                    slot.index = ValueOrFreeIndex::Free(free_head_index);
                } else {
                    panic!("Returning a slot that was already available")
                }
            } else {
                //follow the list until slot with an index greater than the current one is found
                //when one is found, put this new slot between the that slot and the previous one
                let mut prev_slot_index = free_head_index;
                println!("Starting loop");
                loop {
                    match self.slots[prev_slot_index].index {
                        ValueOrFreeIndex::Free(next_index) => {
                            if next_index > slot_key.index.0 {
                                //the new slot is in between the current slot and the prev slot
                                self.slots[prev_slot_index].index =
                                    ValueOrFreeIndex::Free(slot_key.index.0);

                                let slot = &mut self.slots[slot_key.index.0];
                                slot.index = ValueOrFreeIndex::Free(next_index);
                                slot.generation.0 += 1;
                                return;
                            } else {
                                prev_slot_index = next_index;
                            }
                        }
                        ValueOrFreeIndex::End => {
                            self.slots[prev_slot_index].index =
                                ValueOrFreeIndex::Free(slot_key.index.0);
                            //if current head is the end, and the returned slot is greater than it, then, the new slot is the new end
                            let slot = &mut self.slots[slot_key.index.0];
                            slot.generation.0 += 1;
                            slot.index = ValueOrFreeIndex::End;
                            return;
                        }
                        ValueOrFreeIndex::Value(_) => {
                            panic!("Slot free list is pointing to a filled slot")
                        }
                    }
                }
            }
        }
    }

    pub fn free_len(&self) -> usize {
        match self.free_head {
            Some(free_head_index) => {
                let mut length = 0;
                let mut current_index = free_head_index;
                loop {
                    length += 1;
                    match self.slots[current_index].index {
                        ValueOrFreeIndex::Free(next_index) => {
                            if next_index < current_index {
                                panic!("Slot is pointing backwards");
                            } else {
                                current_index = next_index;
                            }
                        }
                        ValueOrFreeIndex::End => {
                            return length;
                        }
                        ValueOrFreeIndex::Value(_) => {
                            panic!("Free list points to a filled slot")
                        }
                    }
                }
            }
            None => return 0,
        }
    }

    pub fn free_slice(&self) -> Vec<usize> {
        let mut free_slice = Vec::new();
        match self.free_head {
            Some(free_head_index) => {
                let mut current_index = free_head_index;
                loop {
                    free_slice.push(current_index);
                    match self.slots[current_index].index {
                        ValueOrFreeIndex::Free(next_index) => {
                            if next_index < current_index {
                                panic!("Slot is pointing backwards");
                            } else {
                                current_index = next_index;
                            }
                        }
                        ValueOrFreeIndex::End => {
                            break;
                        }
                        ValueOrFreeIndex::Value(_) => {
                            panic!("Free list points to a filled slot")
                        }
                    }
                }
            }
            None => {}
        }
        free_slice
    }
}

/// This structure is optimized in the following order iteration > random acess > pushing objects > removing objects
pub struct Slotmap<V> {
    values: Vec<V>,
    /// Array with the slot indexes for the slots that are pointing to a value,
    /// this array has the same order as the Values array
    values_slot: Vec<SlotIndex>,
    slot_collection: SlotCollection,
}

impl<V> Slotmap<V> {
    pub fn with_capacity(capacity: usize) -> Self {
        let values = Vec::<V>::with_capacity(capacity);
        let values_slot = Vec::<SlotIndex>::with_capacity(capacity);
        let slot_collection = SlotCollection::new(capacity);

        Self {
            values,
            values_slot,
            slot_collection,
        }
    }

    pub fn capacity(&self) -> usize {
        self.values.capacity()
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn get_iter(&self) -> Iter<'_, V> {
        self.values.iter()
    }

    pub fn get_iter_mut(&mut self) -> IterMut<'_, V> {
        self.values.iter_mut()
    }

    #[inline]
    pub fn is_valid(&self, key: &SlotKey) -> bool {
        self.slot_collection.is_valid(key)
    }

    pub fn get_value(&self, key: &SlotKey) -> Option<&V> {
        match self.slot_collection.get_value_index(key) {
            Some(index) => Some(&self.values[index]),
            None => None,
        }
    }

    pub fn get_value_mut(&mut self, key: &SlotKey) -> Option<&mut V> {
        match self.slot_collection.get_value_index(key) {
            Some(index) => Some(&mut self.values[index]),
            None => None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn free_list_len(&self) -> usize {
        self.slot_collection.free_len()
    }
    pub fn free_list_slice(&self) -> Vec<usize> {
        self.slot_collection.free_slice()
    }

    /// This function uses the `Vec::reserve_exact` internally to increase the available space
    /// If the `self.len() + aditional` is smaller than `self.capacity()`, then no space is allocated.\
    /// Increased capacity is equatl to `max( 0, self.len() + aditional - self.capacity() )`
    /*pub fn reserve_exact(&mut self, aditional: usize) -> Option<usize> {
        let current_capacity = self.capacity();

        self.values.reserve_exact(aditional);
        self.values_slot.reserve_exact(aditional);

        let new_capacity = self.capacity();

        let extra_capacity = new_capacity - current_capacity;
        if extra_capacity > 0 {
            self.free_list
                .add_free_bucket_to_tail(current_capacity, extra_capacity);
            return Some(extra_capacity);
        }
        None
    }*/

    pub fn remove(&mut self, key: SlotKey) -> Option<V> {
        if self.is_valid(&key) {
            if let ValueOrFreeIndex::Value(value_index) =
                self.slot_collection.slots[key.index.0].index
            {
                if value_index == self.values.len() - 1 || self.values.len() == 1 {
                    let value = self.values.pop().unwrap();
                    let _slot_index = self.values_slot.pop().unwrap();
                    // just needs to return the slot index
                    self.slot_collection.return_slot(key, None);

                    return Some(value);
                } else if self.values.len() > 1 {
                    let value = self.values.swap_remove(value_index);
                    self.values_slot.swap_remove(value_index);
                    // Return slotmap and udapte the value index for the swaped
                    let slot_index = self.values_slot[value_index];
                    self.slot_collection
                        .return_slot(key, Some((slot_index, value_index)));

                    return Some(value);
                } else {
                    unreachable!("The user cannot return a slot to an empty slotmap")
                }
            } else {
                unreachable!("A valid slotkey had a free / end index");
            }
        } else {
            None
        }
    }

    pub fn push(&mut self, value: V) -> Option<SlotKey> {
        if self.values.capacity() == self.values.len() {
            None
        } else {
            //let slot = self.slot_collection.take_slot(self.values.len());
            match self.slot_collection.take_slot(self.values.len()) {
                Some(slot) => {
                    self.values.push(value);
                    self.values_slot.push(slot.index);
                    Some(slot)
                }
                None => None,
            }
        }
    }
}

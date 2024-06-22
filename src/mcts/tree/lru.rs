//! lru implements a Least Recently Used cache for Nodes. This is used to store
//! the nodes of a Tree, allowing arbitrarily long searches as unused memory is
//! continuously freed by the cache.
use std::mem;
use std::ops::Deref;
use std::ops::DerefMut;

use super::Node;

/// Cache is a Least Recently Used (LRU) Cache for [Nodes](Node), which allows
/// the search tree to utilize limited memory efficiently.
#[derive(Clone)]
pub struct Cache {
    map: Vec<Entry>, // Backing storage of the cache.

    void: i32, // Pointer to the first void in the cache.
    head: i32, // Pointer to the most recently used entry.
    tail: i32, // Pointer to the least recently used entry.
}

impl Cache {
    /// new_mib creates a new Cache with the given number of mebibytes of capacity
    /// for storing Nodes. Note that the actual memory usage will be much higher
    /// due to the fact that each Node will have multiple Edges.
    pub fn new_mib(mib: usize) -> Cache {
        Cache::new(1024 * 1024 * mib / mem::size_of::<Entry>())
    }

    /// new creates a new Cache with the given capacity for storing Nodes.
    pub fn new(cap: usize) -> Cache {
        Cache {
            // Initialize the map with the complete void list. The cache can
            // essentially be broken into two mutually exclusive and exhaustive
            // linked lists, one with filled entries and another with void
            // entries. At the start, the entire cache is part of the void list.
            map: (0..cap)
                .map(|i| Entry {
                    val: Default::default(),
                    prev: -1,
                    // Each entry is linked to the next (i + 1) entry. The last
                    // entry where i + 1 = cap has no forward link, so use -1,
                    // which marks the absence of a link.
                    next: if (i + 1) < cap { (i + 1) as i32 } else { -1 },
                })
                .collect(),
            void: 0,  // The first (0) entry is currently a void.
            head: -1, // Currently there is no most recently used entry.
            tail: -1, // Currently there is no least recently used entry.
        }
    }
}

impl Cache {
    /// get returns a reference to the Entry at the given pointer.
    pub fn get(&self, ptr: i32) -> &Entry {
        &self.map[ptr as usize]
    }

    /// get_mut returns a mutable reference to the Entry at the given pointer.
    pub fn get_mut(&mut self, ptr: i32) -> &mut Entry {
        &mut self.map[ptr as usize]
    }

    /// promote makes the given Entry the most recently used one.
    pub fn promote(&mut self, ptr: i32) {
        self.detach(ptr);
        self.attach(ptr);
    }

    /// attach adds the given entry to the non-void list and makes it the head.
    pub fn attach(&mut self, ptr: i32) {
        // Update old head's links if there is one.
        if self.head != -1 {
            self.get_mut(self.head).prev = ptr;
        }

        // Create a copy of the pointer to the old head and make the attached
        // pointer the new head (most recently used entry) of the cache.
        let head_ptr = self.head;
        self.head = ptr;

        // Update the new head's links.
        let node = self.get_mut(ptr);
        node.next = head_ptr;
        node.prev = -1;
    }

    /// push adds the given Node to the cache as its head.
    pub fn push(&mut self, val: Node) -> i32 {
        // Find an Entry to store the node in.
        let node_ptr = if self.void != -1 {
            // Empty entry found, so use that.
            let void = self.void;
            // This pointer will no longer be empty so, update it to
            // the next void spot in the void entry linked list.
            self.void = self.get(self.void).next;
            void
        } else {
            // No void spots left, so purge the least recently used entry (also
            // called the tail of the cache) and use that spot for storage.
            let tail = self.tail; // Store a copy of the current tail.

            // Remove the tail from the cache.
            self.remove_lru(self.tail);

            // Update the links to the newly purged entry to prevent invalid accesses.
            let node = self.get(self.tail);
            let (parent_node, parent_edge) = (node.parent_node, node.parent_edge);
            self.get_mut(parent_node).edge_mut(parent_edge).ptr = -1;

            tail
        };

        // Update the value of the entry and attach it to the cache.
        self.get_mut(node_ptr).val = val;
        self.attach(node_ptr);

        // Return the pointer to the newly added entry.
        node_ptr
    }

    /// detach removes the given entry from the cache, but keeps its data. To
    /// also remove the entry's data, use [`Self::remove_lru`].
    fn detach(&mut self, ptr: i32) {
        let node = self.get_mut(ptr);
        let (prev_ptr, next_ptr) = (node.prev, node.next);

        // Update the links for the detached node.
        node.prev = -1;
        node.next = -1;

        // Update the links for the Node's predecessor, if any.
        if prev_ptr != -1 {
            let prev = self.get_mut(prev_ptr);
            prev.next = next_ptr;
        } else {
            // If there is no predecessor, this was the head node.
            // Update the pointer to the head node to the successor.
            self.head = next_ptr;
        }

        // Update the links for the Node's successor, if any.
        if next_ptr != -1 {
            let next = self.get_mut(next_ptr);
            next.prev = prev_ptr;
        } else {
            // If there is no successor, this was the tail node.
            // Update the pointer to the tail node to the predecessor.
            self.tail = prev_ptr;
        }
    }

    /// detach removes the given entry and its data from the cache. To keep the
    /// data, use [`Self::detach`] instead.
    fn remove_lru(&mut self, ptr: i32) {
        self.detach(ptr); // Detach the given entry.
        self.get_mut(ptr).val = Default::default(); // Purge its data.
    }
}

/// Entry is one of the entries in the LRU [Cache]. Externally, it is mainly
/// used by dereferencing it into a [Node] instead of directly using it.
#[derive(Clone)]
pub struct Entry {
    val: Node,
    prev: i32,
    next: i32,
}

impl Deref for Entry {
    type Target = Node;

    /// Entry can be dereferenced into its Node value.
    fn deref(&self) -> &Self::Target {
        &self.val
    }
}

impl DerefMut for Entry {
    /// Entry can be mutably dereferenced into its Node value.
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.val
    }
}

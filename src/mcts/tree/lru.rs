use core::fmt;

use std::mem;
use std::ops::Deref;
use std::ops::DerefMut;

use super::Node;

extern crate alloc;

#[derive(Clone)]
pub struct Entry {
    val: Node,
    prev: i32,
    next: i32,
}

impl Deref for Entry {
    type Target = Node;
    fn deref(&self) -> &Self::Target {
        &self.val
    }
}

impl DerefMut for Entry {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.val
    }
}

#[derive(Clone)]
pub struct Cache {
    map: Vec<Entry>,
    cap: usize,

    void: i32,
    head: i32,
    tail: i32,
}

impl Cache {
    pub fn new_mib(mib: usize) -> Cache {
        Cache::new(1024 * 1024 * mib / mem::size_of::<Node>())
    }
    pub fn new(cap: usize) -> Cache {
        Cache {
            map: (0..cap)
                .map(|i| Entry {
                    val: Default::default(),
                    prev: -1,
                    // The last element is not linked to anything
                    next: if (i + 1) < cap { (i + 1) as i32 } else { -1 },
                })
                .collect(),
            cap,
            void: 0,
            head: -1,
            tail: -1,
        }
    }
}

impl Cache {
    pub fn get(&self, k: i32) -> &Entry {
        &self.map[k as usize]
    }

    pub fn get_mut(&mut self, k: i32) -> &mut Entry {
        &mut self.map[k as usize]
    }

    pub fn promote(&mut self, node_ptr: i32) {
        self.detach(node_ptr);
        self.attach(node_ptr);
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn cap(&self) -> usize {
        self.cap
    }

    fn detach(&mut self, k: i32) {
        let node = self.get_mut(k);
        let (prev_ptr, next_ptr) = (node.prev, node.next);
        node.prev = -1;
        node.next = -1;

        if prev_ptr != -1 {
            let prev = self.get_mut(prev_ptr);
            prev.next = next_ptr;
        } else {
            self.head = next_ptr;
        }

        if next_ptr != -1 {
            let next = self.get_mut(next_ptr);
            next.prev = prev_ptr;
        } else {
            self.tail = prev_ptr;
        }
    }

    pub fn attach(&mut self, node_ptr: i32) {
        let head_ptr = self.head;
        if head_ptr != -1 {
            let head = self.get_mut(head_ptr);
            head.prev = node_ptr;
        }

        self.head = node_ptr;

        let node = self.get_mut(node_ptr);

        node.next = head_ptr;
        node.prev = -1;
    }

    pub fn push(&mut self, val: Node) -> i32 {
        let node_ptr = if self.void != -1 {
            let void = self.void;
            self.void = self.get(self.void).next;
            void
        } else {
            let tail = self.tail;

            self.remove_lru(self.tail);

            let node = self.get(self.tail);
            let (parent_node, parent_edge) = (node.parent_node, node.parent_edge);
            self.get_mut(parent_node).edge_mut(parent_edge).ptr = -1;

            tail
        };

        let node = self.get_mut(node_ptr);
        node.val = val;
        self.attach(node_ptr);

        node_ptr
    }

    fn remove_lru(&mut self, ptr: i32) {
        self.detach(ptr);
        self.get_mut(ptr).val = Default::default();
    }
}

impl fmt::Debug for Cache {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("LruCache")
            .field("len", &self.len())
            .field("cap", &self.cap())
            .finish()
    }
}

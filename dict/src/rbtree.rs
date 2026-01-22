use crate::dict_allocator::{DictAllocator, DictBox};
use crate::dict_string::DictString;
use std::ptr::NonNull;

#[repr(C)]
pub struct RedBlackTree {
    root: NonNull<RedBlackTreeNode>,
    nil: NonNull<RedBlackTreeNode>,
}

#[repr(C)]
pub struct RedBlackTreeNode {
    key: u64,
    value: DictString,
    left: NonNull<RedBlackTreeNode>,
    right: NonNull<RedBlackTreeNode>,
    parent: NonNull<RedBlackTreeNode>,
    color: RedBlackTreeNodeColor,
}

#[repr(u8)]
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum RedBlackTreeNodeColor {
    Red,
    Black,
}

// Internal functions
impl RedBlackTree {
    unsafe fn internal_minimum(
        t: &RedBlackTree,
        x: NonNull<RedBlackTreeNode>,
    ) -> NonNull<RedBlackTreeNode> {
        unsafe {
            let mut x = x;
            while x.as_ref().left != t.nil {
                x = x.as_ref().left;
            }
            x
        }
    }

    unsafe fn internal_maximum(
        t: &RedBlackTree,
        x: NonNull<RedBlackTreeNode>,
    ) -> NonNull<RedBlackTreeNode> {
        unsafe {
            let mut x = x;
            while x.as_ref().right != t.nil {
                x = x.as_ref().right;
            }
            x
        }
    }

    // unsafe fn internal_search(
    //     t: &RedBlackTree,
    //     x: NonNull<RedBlackTreeNode>,
    //     key: u64,
    // ) -> NonNull<RedBlackTreeNode> {
    //     unsafe {
    //         if x == t.nil || key == x.as_ref().key {
    //             return x;
    //         }
    //         if key < x.as_ref().key {
    //             return RedBlackTree::internal_search(t, x.as_ref().left, key);
    //         }
    //         RedBlackTree::internal_search(t, x.as_ref().right, key)
    //     }
    // }

    unsafe fn internal_iterative_search(
        t: &RedBlackTree,
        x: NonNull<RedBlackTreeNode>,
        key: u64,
    ) -> NonNull<RedBlackTreeNode> {
        unsafe {
            let mut x = x;
            while x != t.nil && key != x.as_ref().key {
                if key < x.as_ref().key {
                    x = x.as_ref().left;
                } else {
                    x = x.as_ref().right;
                }
            }
            x
        }
    }

    // unsafe fn internal_successor(
    //     t: &RedBlackTree,
    //     x: NonNull<RedBlackTreeNode>,
    // ) -> NonNull<RedBlackTreeNode> {
    //     unsafe {
    //         if x.as_ref().right != t.nil {
    //             return RedBlackTree::internal_minimum(t, x.as_ref().right);
    //         }
    //         let mut x = x;
    //         let mut y = x.as_ref().parent;
    //         while y != t.nil && x == y.as_ref().right {
    //             x = y;
    //             y = y.as_ref().parent;
    //         }
    //         y
    //     }
    // }
    //
    // unsafe fn internal_predecessor(
    //     t: &RedBlackTree,
    //     x: NonNull<RedBlackTreeNode>,
    // ) -> NonNull<RedBlackTreeNode> {
    //     unsafe {
    //         if x.as_ref().left != t.nil {
    //             return RedBlackTree::internal_maximum(t, x.as_ref().left);
    //         }
    //         let mut x = x;
    //         let mut y = x.as_ref().parent;
    //         while y != t.nil && x == y.as_ref().left {
    //             x = y;
    //             y = y.as_ref().parent;
    //         }
    //         y
    //     }
    // }

    /// Rotates the tree left around node `x`.
    /// Assumes `x.right` is not NIL.
    unsafe fn rotate_left(t: &mut RedBlackTree, mut x: NonNull<RedBlackTreeNode>) {
        unsafe {
            // init y
            let mut y = x.as_ref().right;

            // Turn y's left subtree into x's right subtree
            x.as_mut().right = y.as_ref().left;
            if y.as_ref().left != t.nil {
                y.as_mut().left.as_mut().parent = x;
            }

            // Link x's parent to y
            y.as_mut().parent = x.as_ref().parent;
            if x.as_ref().parent == t.nil {
                t.root = y;
            } else if x == x.as_ref().parent.as_ref().left {
                x.as_mut().parent.as_mut().left = y;
            } else {
                x.as_mut().parent.as_mut().right = y;
            }

            // Put x on y's left
            y.as_mut().left = x;
            x.as_mut().parent = y;
        }
    }

    /// Rotates the tree left around node `y`.
    /// Assumes `y.left` is not NIL.
    unsafe fn rotate_right(t: &mut RedBlackTree, mut y: NonNull<RedBlackTreeNode>) {
        unsafe {
            // init x
            let mut x = y.as_ref().left;

            // Turn x's right subtree into y's right subtree
            y.as_mut().left = x.as_ref().right;
            if x.as_ref().right != t.nil {
                x.as_mut().right.as_mut().parent = y;
            }

            // Link y's parent to x
            x.as_mut().parent = y.as_ref().parent;
            if y.as_ref().parent == t.nil {
                t.root = x;
            } else if y == y.as_ref().parent.as_ref().right {
                y.as_mut().parent.as_mut().right = x;
            } else {
                y.as_mut().parent.as_mut().left = x;
            }

            // Put y on x's right
            x.as_mut().right = y;
            y.as_mut().parent = x;
        }
    }

    unsafe fn internal_insert(t: &mut RedBlackTree, mut z: NonNull<RedBlackTreeNode>) {
        unsafe {
            let mut y = t.nil;
            let mut x = t.root;

            while x != t.nil {
                y = x;
                if z.as_ref().key < x.as_ref().key {
                    x = x.as_ref().left;
                } else {
                    x = x.as_ref().right;
                }
            }
            z.as_mut().parent = y;
            if y == t.nil {
                t.root = z;
            } else if z.as_ref().key < y.as_ref().key {
                y.as_mut().left = z;
            } else {
                y.as_mut().right = z;
            }
            z.as_mut().left = t.nil;
            z.as_mut().right = t.nil;
            z.as_mut().color = RedBlackTreeNodeColor::Red;
            RedBlackTree::insert_fixup(t, z);
        }
    }

    unsafe fn insert_fixup(t: &mut RedBlackTree, mut z: NonNull<RedBlackTreeNode>) {
        unsafe {
            while z.as_ref().parent.as_ref().color == RedBlackTreeNodeColor::Red {
                if z.as_ref().parent == z.as_ref().parent.as_ref().parent.as_ref().left {
                    z = RedBlackTree::insert_fixup_left(t, z);
                } else {
                    z = RedBlackTree::insert_fixup_right(t, z);
                }
            }
            t.root.as_mut().color = RedBlackTreeNodeColor::Black;
        }
    }

    unsafe fn insert_fixup_left(
        t: &mut RedBlackTree,
        mut z: NonNull<RedBlackTreeNode>,
    ) -> NonNull<RedBlackTreeNode> {
        unsafe {
            let mut y = z.as_ref().parent.as_ref().parent.as_ref().right;
            if y.as_ref().color == RedBlackTreeNodeColor::Red {
                z.as_mut().parent.as_mut().color = RedBlackTreeNodeColor::Black;
                y.as_mut().color = RedBlackTreeNodeColor::Black;
                z.as_mut().parent.as_mut().parent.as_mut().color = RedBlackTreeNodeColor::Red;
                z.as_ref().parent.as_ref().parent
            } else {
                if z == z.as_ref().parent.as_ref().right {
                    z = z.as_ref().parent;
                    RedBlackTree::rotate_left(t, z);
                }
                z.as_mut().parent.as_mut().color = RedBlackTreeNodeColor::Black;
                z.as_mut().parent.as_mut().parent.as_mut().color = RedBlackTreeNodeColor::Red;
                RedBlackTree::rotate_right(t, z.as_ref().parent.as_ref().parent);
                z
            }
        }
    }

    unsafe fn insert_fixup_right(
        t: &mut RedBlackTree,
        mut z: NonNull<RedBlackTreeNode>,
    ) -> NonNull<RedBlackTreeNode> {
        unsafe {
            let mut y = z.as_ref().parent.as_ref().parent.as_ref().left;
            if y.as_ref().color == RedBlackTreeNodeColor::Red {
                z.as_mut().parent.as_mut().color = RedBlackTreeNodeColor::Black;
                y.as_mut().color = RedBlackTreeNodeColor::Black;
                z.as_mut().parent.as_mut().parent.as_mut().color = RedBlackTreeNodeColor::Red;
                z.as_ref().parent.as_ref().parent
            } else {
                if z == z.as_ref().parent.as_ref().left {
                    z = z.as_ref().parent;
                    RedBlackTree::rotate_right(t, z);
                }
                z.as_mut().parent.as_mut().color = RedBlackTreeNodeColor::Black;
                z.as_mut().parent.as_mut().parent.as_mut().color = RedBlackTreeNodeColor::Red;
                RedBlackTree::rotate_left(t, z.as_ref().parent.as_ref().parent);
                z
            }
        }
    }

    unsafe fn transplant(
        t: &mut RedBlackTree,
        mut u: NonNull<RedBlackTreeNode>,
        mut v: NonNull<RedBlackTreeNode>,
    ) {
        unsafe {
            if u.as_ref().parent == t.nil {
                t.root = v;
            } else if u == u.as_ref().parent.as_ref().left {
                u.as_mut().parent.as_mut().left = v;
            } else {
                u.as_mut().parent.as_mut().right = v;
            }
            v.as_mut().parent = u.as_ref().parent;
            v.as_mut().parent = u.as_ref().parent;
        }
    }

    unsafe fn delete(t: &mut RedBlackTree, z: NonNull<RedBlackTreeNode>) {
        unsafe {
            let mut y = z;
            let mut x;
            let mut y_original_color = y.as_ref().color;
            if z.as_ref().left == t.nil {
                x = z.as_ref().right;
                RedBlackTree::transplant(t, z, z.as_ref().right);
            } else if z.as_ref().right == t.nil {
                x = z.as_ref().left;
                RedBlackTree::transplant(t, z, z.as_ref().left);
            } else {
                y = RedBlackTree::internal_minimum(t, z.as_ref().right);
                y_original_color = y.as_ref().color;
                x = y.as_ref().right;
                Self::delete_two_children_case(t, z, y, &mut x);
            }
            if y_original_color == RedBlackTreeNodeColor::Black {
                RedBlackTree::delete_fixup(t, x)
            }

            let allocator = DictAllocator::new();
            allocator.dealloc(z.as_ptr());
        }
    }

    unsafe fn delete_two_children_case(
        t: &mut RedBlackTree,
        z: NonNull<RedBlackTreeNode>,
        mut y: NonNull<RedBlackTreeNode>,
        x: &mut NonNull<RedBlackTreeNode>,
    ) {
        unsafe {
            if y.as_ref().parent == z {
                x.as_mut().parent = y;
            } else {
                RedBlackTree::transplant(t, y, y.as_ref().right);
                y.as_mut().right = z.as_ref().right;
                y.as_mut().right.as_mut().parent = y;
            }
            RedBlackTree::transplant(t, z, y);
            y.as_mut().left = z.as_ref().left;
            y.as_mut().left.as_mut().parent = y;
            y.as_mut().color = z.as_ref().color;
        }
    }

    unsafe fn delete_fixup(t: &mut RedBlackTree, mut x: NonNull<RedBlackTreeNode>) {
        unsafe {
            while x != t.root && x.as_ref().color == RedBlackTreeNodeColor::Black {
                if x == x.as_ref().parent.as_ref().left {
                    x = RedBlackTree::delete_fixup_left(t, x);
                } else {
                    x = RedBlackTree::delete_fixup_right(t, x);
                }
            }
            x.as_mut().color = RedBlackTreeNodeColor::Black;
        }
    }

    unsafe fn delete_fixup_left(
        t: &mut RedBlackTree,
        mut x: NonNull<RedBlackTreeNode>,
    ) -> NonNull<RedBlackTreeNode> {
        unsafe {
            let mut w = x.as_ref().parent.as_ref().right;
            if w.as_ref().color == RedBlackTreeNodeColor::Red {
                w.as_mut().color = RedBlackTreeNodeColor::Black;
                x.as_mut().parent.as_mut().color = RedBlackTreeNodeColor::Red;
                RedBlackTree::rotate_left(t, x.as_ref().parent);
                w = x.as_ref().parent.as_ref().right;
            }
            if w.as_ref().left.as_ref().color == RedBlackTreeNodeColor::Black
                && w.as_ref().right.as_ref().color == RedBlackTreeNodeColor::Black
            {
                w.as_mut().color = RedBlackTreeNodeColor::Red;
                x = x.as_ref().parent;
            } else {
                if w.as_ref().right.as_ref().color == RedBlackTreeNodeColor::Black {
                    w.as_mut().left.as_mut().color = RedBlackTreeNodeColor::Black;
                    w.as_mut().color = RedBlackTreeNodeColor::Red;
                    RedBlackTree::rotate_right(t, w);
                    w = x.as_ref().parent.as_ref().right;
                }
                w.as_mut().color = x.as_ref().parent.as_ref().color;
                x.as_mut().parent.as_mut().color = RedBlackTreeNodeColor::Black;
                w.as_mut().right.as_mut().color = RedBlackTreeNodeColor::Black;
                RedBlackTree::rotate_left(t, x.as_ref().parent);
                x = t.root;
            }
            x
        }
    }

    unsafe fn delete_fixup_right(
        t: &mut RedBlackTree,
        mut x: NonNull<RedBlackTreeNode>,
    ) -> NonNull<RedBlackTreeNode> {
        unsafe {
            let mut w = x.as_ref().parent.as_ref().left;
            if w.as_ref().color == RedBlackTreeNodeColor::Red {
                w.as_mut().color = RedBlackTreeNodeColor::Black;
                x.as_mut().parent.as_mut().color = RedBlackTreeNodeColor::Red;
                RedBlackTree::rotate_right(t, x.as_ref().parent);
                w = x.as_ref().parent.as_ref().left;
            }
            if w.as_ref().right.as_ref().color == RedBlackTreeNodeColor::Black
                && w.as_ref().left.as_ref().color == RedBlackTreeNodeColor::Black
            {
                w.as_mut().color = RedBlackTreeNodeColor::Red;
                x = x.as_ref().parent;
            } else {
                if w.as_ref().left.as_ref().color == RedBlackTreeNodeColor::Black {
                    w.as_mut().right.as_mut().color = RedBlackTreeNodeColor::Black;
                    w.as_mut().color = RedBlackTreeNodeColor::Red;
                    RedBlackTree::rotate_left(t, w);
                    w = x.as_ref().parent.as_ref().left;
                }
                w.as_mut().color = x.as_ref().parent.as_ref().color;
                x.as_mut().parent.as_mut().color = RedBlackTreeNodeColor::Black;
                w.as_mut().left.as_mut().color = RedBlackTreeNodeColor::Black;
                RedBlackTree::rotate_right(t, x.as_ref().parent);
                x = t.root;
            }
            x
        }
    }

    unsafe fn drop_node(t: &mut RedBlackTree, node: NonNull<RedBlackTreeNode>) {
        if node != t.nil {
            unsafe {
                RedBlackTree::drop_node(t, node.as_ref().left);
                RedBlackTree::drop_node(t, node.as_ref().right);
            }
            let allocator = DictAllocator::new();
            allocator.dealloc(node.as_ptr());
        }
    }
}

// Rust Public Interface
impl RedBlackTree {
    pub fn new() -> Option<RedBlackTree> {
        unsafe {
            let nil_node = RedBlackTreeNode {
                key: 0,
                value: DictString::new(),
                left: NonNull::dangling(),
                right: NonNull::dangling(),
                parent: NonNull::dangling(),
                color: RedBlackTreeNodeColor::Black,
            };

            let nil_ptr = DictBox::new(nil_node);
            let mut nil = NonNull::new_unchecked(nil_ptr?.into_raw());

            nil.as_mut().left = nil;
            nil.as_mut().right = nil;
            nil.as_mut().parent = nil;

            Some(RedBlackTree { root: nil, nil })
        }
    }

    pub fn insert(&mut self, key: u64, value: DictString) -> bool {
        unsafe {
            let mut node = RedBlackTree::internal_iterative_search(self, self.root, key);
            if node != self.nil {
                node.as_mut().value = value;
                true
            } else {
                let new_node = RedBlackTreeNode {
                    key,
                    value,
                    left: self.nil,
                    right: self.nil,
                    parent: self.nil,
                    color: RedBlackTreeNodeColor::Red,
                };

                if let Some(boxed) = DictBox::new(new_node) {
                    let ptr = NonNull::new_unchecked(boxed.into_raw());
                    RedBlackTree::internal_insert(self, ptr);
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn find(&self, key: u64) -> Option<&DictString> {
        unsafe {
            let node = RedBlackTree::internal_iterative_search(self, self.root, key);
            if node != self.nil {
                Some(&node.as_ref().value)
            } else {
                None
            }
        }
    }

    pub fn minimum(&self) -> Option<&DictString> {
        unsafe {
            let node = RedBlackTree::internal_minimum(self, self.root);
            if node != self.nil {
                Some(&node.as_ref().value)
            } else {
                None
            }
        }
    }

    pub fn maximum(&self) -> Option<&DictString> {
        unsafe {
            let node = RedBlackTree::internal_maximum(self, self.root);
            if node != self.nil {
                Some(&node.as_ref().value)
            } else {
                None
            }
        }
    }

    pub fn contains(&self, key: u64) -> bool {
        unsafe {
            let node = RedBlackTree::internal_iterative_search(self, self.root, key);
            node != self.nil
        }
    }

    pub fn remove(&mut self, key: u64) -> bool {
        unsafe {
            let node = RedBlackTree::internal_iterative_search(self, self.root, key);
            if node != self.nil {
                RedBlackTree::delete(self, node);
                true
            } else {
                false
            }
        }
    }
}

#[macro_export]
macro_rules! rbt {
    ( $( $key:expr => $val:expr ),* $(,)? ) => {
        {
            let mut tree_opt = RedBlackTree::new();
            if let Some(tree) = &mut tree_opt {
                $(
                let _ = tree.insert($key, $val.into());
                )*
            }
            tree_opt
        }
    };
}

impl Drop for RedBlackTree {
    fn drop(&mut self) {
        unsafe {
            RedBlackTree::drop_node(self, self.root);
            // Free the sentinel at the end
            let allocator = DictAllocator::new();
            allocator.dealloc(self.nil.as_ptr());
        }
    }
}

// C Public Interface
#[unsafe(no_mangle)]
pub extern "C" fn rbt_new() -> *mut RedBlackTree {
    match RedBlackTree::new() {
        Some(tree) => {
            if let Some(boxed) = DictBox::new(tree) {
                boxed.into_raw()
            } else {
                std::ptr::null_mut()
            }
        }
        None => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rbt_free(ptr: *mut RedBlackTree) {
    if !ptr.is_null() {
        let allocator = DictAllocator::new();
        allocator.dealloc(ptr);
    }
}

/// # Safety
/// The [ptr] must be NULL or point to a valid [RedBlackTree]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rbt_insert(
    ptr: *mut RedBlackTree,
    key: u64,
    value: *const libc::c_char,
) -> bool {
    unsafe {
        if ptr.is_null() || value.is_null() {
            return false;
        }

        let tree = &mut *ptr;
        let c_str = std::ffi::CStr::from_ptr(value);
        let bytes = c_str.to_bytes();

        if let Some(ds) = DictString::from(bytes) {
            tree.insert(key, ds)
        } else {
            false
        }
    }
}

/// # Safety
/// The [ptr] must be NULL or point to a valid [RedBlackTree]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rbt_find(ptr: *const RedBlackTree, key: u64) -> *const libc::c_char {
    unsafe {
        if ptr.is_null() {
            return std::ptr::null();
        }

        let tree = &*ptr;
        match tree.find(key) {
            Some(ds) => ds.as_str().as_ptr() as *const libc::c_char,
            None => std::ptr::null(),
        }
    }
}

/// # Safety
/// The [ptr] must be NULL or point to a valid [RedBlackTree]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rbt_contains(ptr: *const RedBlackTree, key: u64) -> bool {
    unsafe {
        if ptr.is_null() {
            return false;
        }

        let tree = &*ptr;
        tree.find(key).is_some()
    }
}

/// # Safety
/// The [ptr] must be NULL or point to a valid [RedBlackTree]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rbt_minimum(ptr: *const RedBlackTree) -> *const libc::c_char {
    unsafe {
        if ptr.is_null() {
            return std::ptr::null();
        }

        let tree = &*ptr;
        match tree.minimum() {
            Some(ds) => ds.as_str().as_ptr() as *const libc::c_char,
            None => std::ptr::null(),
        }
    }
}

/// # Safety
/// The [ptr] must be NULL or point to a valid [RedBlackTree]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rbt_maximum(ptr: *const RedBlackTree) -> *const libc::c_char {
    unsafe {
        if ptr.is_null() {
            return std::ptr::null();
        }

        let tree = &*ptr;
        match tree.maximum() {
            Some(ds) => ds.as_str().as_ptr() as *const libc::c_char,
            None => std::ptr::null(),
        }
    }
}

/// # Safety
/// The [ptr] must be NULL or point to a valid [RedBlackTree]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rbt_remove(ptr: *mut RedBlackTree, key: u64) -> bool {
    unsafe {
        if ptr.is_null() {
            return false;
        }

        let tree = &mut *ptr;
        tree.remove(key)
    }
}

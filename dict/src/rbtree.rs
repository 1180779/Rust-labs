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

impl RedBlackTree {
    /// Creates a new Red-Black Tree with a sentinel (NIL) node.
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

    unsafe fn minimum(t: &RedBlackTree, x: NonNull<RedBlackTreeNode>) -> NonNull<RedBlackTreeNode> {
        unsafe {
            let mut x = x;
            while x.as_ref().left != t.nil {
                x = x.as_ref().left;
            }
            x
        }
    }

    unsafe fn maximum(t: &RedBlackTree, x: NonNull<RedBlackTreeNode>) -> NonNull<RedBlackTreeNode> {
        unsafe {
            let mut x = x;
            while x.as_ref().right != t.nil {
                x = x.as_ref().right;
            }
            x
        }
    }

    unsafe fn search(
        t: &RedBlackTree,
        x: NonNull<RedBlackTreeNode>,
        key: u64,
    ) -> NonNull<RedBlackTreeNode> {
        unsafe {
            if x == t.nil || key == x.as_ref().key {
                return x;
            }
            if key < x.as_ref().key {
                return RedBlackTree::search(t, x.as_ref().left, key);
            }
            RedBlackTree::search(t, x.as_ref().right, key)
        }
    }

    unsafe fn iterative_search(
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

    unsafe fn successor(
        t: &RedBlackTree,
        x: NonNull<RedBlackTreeNode>,
    ) -> NonNull<RedBlackTreeNode> {
        unsafe {
            if x.as_ref().right != t.nil {
                return RedBlackTree::minimum(t, x.as_ref().right);
            }
            let mut x = x;
            let mut y = x.as_ref().parent;
            while y != t.nil && x == y.as_ref().right {
                x = y;
                y = y.as_ref().parent;
            }
            y
        }
    }

    unsafe fn predecessor(
        t: &RedBlackTree,
        x: NonNull<RedBlackTreeNode>,
    ) -> NonNull<RedBlackTreeNode> {
        unsafe {
            if x.as_ref().left != t.nil {
                return RedBlackTree::maximum(t, x.as_ref().left);
            }
            let mut x = x;
            let mut y = x.as_ref().parent;
            while y != t.nil && x == y.as_ref().left {
                x = y;
                y = y.as_ref().parent;
            }
            y
        }
    }

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

    unsafe fn insert(t: &mut RedBlackTree, mut z: NonNull<RedBlackTreeNode>) {
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
            if y.as_ref().parent == t.nil {
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
                y = RedBlackTree::minimum(t, z.as_ref().right);
                y_original_color = y.as_ref().color;
                x = y.as_ref().right;
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
            if y_original_color == RedBlackTreeNodeColor::Black {
                RedBlackTree::delete_fixup(t, x)
            }

            let allocator = DictAllocator::new();
            allocator.dealloc(z.as_ptr());
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

impl RedBlackTree {
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

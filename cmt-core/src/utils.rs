//! module to store the utility functions of CMT
use crate::{Hash, TreeNode};
use sha2::{Digest, Sha256};

pub fn calculate_merkle_hash<K: AsRef<[u8]>>(
    key: &K,
    left_child_hash: &Hash,
    right_child_hash: &Hash,
) -> Hash {
    let mut buf = Vec::new();
    buf.extend_from_slice(key.as_ref());
    if left_child_hash < right_child_hash {
        buf.extend_from_slice(left_child_hash);
        buf.extend_from_slice(right_child_hash);
    } else {
        buf.extend_from_slice(right_child_hash);
        buf.extend_from_slice(left_child_hash);
    }
    let mut hasher = Sha256::new();
    hasher.update(&buf);
    hasher.finalize().to_vec()
}

pub fn rotate_left<K: AsRef<[u8]>, V>(mut x: Box<TreeNode<K, V>>) -> Box<TreeNode<K, V>> {
    let mut y = x.right.take().expect("rotate_left requires right child");

    // move y.left into x.right
    x.right = y.left.take();

    // recompute x.hash
    let left_hash = x.left.as_ref().map(|n| n.hash.clone()).unwrap_or_default();
    let right_hash = x.right.as_ref().map(|n| n.hash.clone()).unwrap_or_default();
    x.hash = calculate_merkle_hash(&x.key, &left_hash, &right_hash);

    // put x as left child of y
    y.left = Some(x);

    // recompute y.hash
    let left_hash = y.left.as_ref().map(|n| n.hash.clone()).unwrap_or_default();
    let right_hash = y.right.as_ref().map(|n| n.hash.clone()).unwrap_or_default();
    y.hash = calculate_merkle_hash(&y.key, &left_hash, &right_hash);

    y
}

pub fn rotate_right<K: AsRef<[u8]>, V>(mut y: Box<TreeNode<K, V>>) -> Box<TreeNode<K, V>> {
    let mut x = y.left.take().expect("rotate_right requires left child");

    // move x.right into y.left
    y.left = x.right.take();

    // recompute y.hash
    let left_hash = y.left.as_ref().map(|n| n.hash.clone()).unwrap_or_default();
    let right_hash = y.right.as_ref().map(|n| n.hash.clone()).unwrap_or_default();
    y.hash = calculate_merkle_hash(&y.key, &left_hash, &right_hash);

    // put y as right child of x
    x.right = Some(y);

    // recompute x.hash
    let left_hash = x.left.as_ref().map(|n| n.hash.clone()).unwrap_or_default();
    let right_hash = x.right.as_ref().map(|n| n.hash.clone()).unwrap_or_default();
    x.hash = calculate_merkle_hash(&x.key, &left_hash, &right_hash);

    x
}

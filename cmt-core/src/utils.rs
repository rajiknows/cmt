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

// Let CMT proof be a proof of membership of an element e in a tree T, represented as proof =
// [prefix, suffix], where:
// • prefix is an ordered list of Merkle path nodes, each containing pairs of values (n.e.k, n.mh) for
// each node n;
// • suffix consists of e.leftChildMH and e.rightChildMH, representing the subtree structure of e;
// • existence is a boolean flag indicating whether e exists in the tree;
// • nonExistenceKey is used when e does not exist in the tree, and helps verify that e is absent.
// The initial value of acc is computed as:
// acc = H((existence?e.k : nonExistenceKey) ∥ proof.suffix[0] ∥ proof.suffix[1])
// ensuring that proof.suffix[0] < proof.suffix[1].
// Then, acc is iteratively updated using values from prefix:
// (
// acc = H(n.e.k ∥ n.mh ∥ acc), if n.mh < acc,
// acc = H(n.e.k ∥ acc ∥ n.mh), otherwise.
// The proof is considered valid if the final value of acc matches the root of T.
//
// Algortithm
//
// Input: Element e = k to be proven in tree T
// Output: Proof proof = [prefix, suffix, existence, nonExistenceKey]
// Function GenerateProof(T, e):
// Initialize empty lists: prefix ← [], suffix ← [];
// Initialize bool variable existence ← true;
// Initialize variable currentNode ← null;
// /* Get the appropriate node for the entry e */
// if e does not exist in the tree T then
// currentNode ← node with appropriate key for non-existence proof;
// existence ← false;
// nonExistenceKey ← currentNode.e.k;
// else
// currentNode ← node in T where n.e = e;
// /* Set suffix as the hash values of currentNode’s children */
// suffix ← [n.leftChildMH, n.rightChildMH];
// /* Construct prefix by traversing the path to the root */
// while currentNode is not root do
// parent ← Parent(currentNode);
// Append (parent.e.k, parent.mh) to prefix;
// currentNode ← parent;
// return [prefix, suffix, existence, nonExistenceKey];

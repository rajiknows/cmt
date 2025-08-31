
# Cartesian Merkle Tree (CMT) in Rust

A **Cartesian Merkle Tree (CMT)** is a data structure that combines:
- A **treap** (binary search tree + heap, with priority = hash(key))
- A **Merkle tree** (cryptographic commitments at each node)

This makes it:
- **Deterministic** – same set of keys → same tree shape  
- **Efficient** – `O(log n)` inserts, updates, deletes  
- **Proof-friendly** – compact membership and non-membership proofs  
- **Lightweight** – `n` storage (vs ~2n for classic Merkle trees)  

📄 Reference: *“Cartesian Merkle Trees”* (2025)  

---

## Project Structure
cartesian-merkle-tree/
├── cmt-core/ # nodes, hash trait, proofs
├── cmt-concurrent/ # atomics, epoch-based reclamation, lock-free rotations
├── cmt-bench/ # criterion benchmarks
└── cmt-examples/ # demos (airdrop list, allowlist, range queries)




---

## Roadmap (14-Day Sprint)

**Week 1 – Core (single-threaded)**
1. Setup repo, define API
2. Implement deterministic treap insert
3. Add Merkle hashing to nodes
4. Membership proofs
5. Non-membership proofs
6. Delete + update
7. Benchmarks vs Sparse Merkle Tree

**Week 2 – Concurrent + Advanced**
8. Add epoch-based memory reclamation
9. Atomic nodes (CAS updates)
10. Lock-free insert
11. Lock-free delete
12. Proof verification under concurrency
13. Benchmarks vs SMT & Crossbeam structures
14. Wrap-up + blog post

---

## Examples

Run demo:

```bash
```
```
```
cargo run -p cmt-examples
```
```

## Benchmarks

Run benchmarks:

```
```
cargo bench -p cmt-bench
```
```

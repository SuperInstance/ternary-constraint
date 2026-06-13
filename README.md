# ternary-constraint

Constraint satisfaction and propagation for ternary variables. Implements ternary domains (subsets of {-1, 0, +1}), binary and unary constraints, AllDifferent, **AC-3 arc consistency**, **backtracking search** with forward checking and MRV variable ordering, and a ternary N-Queens solver.

## Why It Matters

Many agent coordination problems reduce to constraint satisfaction: assign values to variables such that all constraints hold. When variables are ternary (each can be -1, 0, or +1), the search space is $3^n$ rather than $2^n$ — larger, but richer, because the "unknown" value captures genuine ambiguity.

This crate provides industrial-grade CSP solving for ternary variable systems:

- **Arc consistency (AC-3)**: Prune impossible values before searching.
- **Backtracking with forward checking**: Avoid dead branches early.
- **MRV heuristic**: Assign the most constrained variable first.

Within the **γ + η = C** framework:

| Symbol | Domain |
|--------|--------|
| γ | `TernaryDomain` — possible values for each variable |
| η | Search strategy: AC-3 propagation, variable ordering, backtracking |
| C | `Constraint` set — binary, unary, AllDifferent relations |

## How It Works

### Ternary Domains

Each variable has a domain that is a **subset** of {-1, 0, +1}. Unlike traditional CSPs where domains may contain hundreds of values, ternary domains have at most 3 — making domain operations O(1):

| Operation | Complexity |
|-----------|------------|
| `contains(v)` | O(1) |
| `remove(v)` | O(1) |
| `intersect(other)` | O(1) |
| `union(other)` | O(1) |
| `size()` | O(1) |

This means arc consistency and forward checking are **extremely fast** per-variable, making ternary CSPs solvable even with hundreds of variables.

### AC-3 Algorithm

AC-3 enforces **arc consistency**: for every constraint between variables $X_i$ and $X_j$, every value in $D(X_i)$ must have at least one supporting value in $D(X_j)$.

**Algorithm:**

1. Initialize a queue with all constraint arcs $(X_i, X_j)$ and $(X_j, X_i)$.
2. Pop arc $(X_i, X_j)$. **Revise**: remove any value $v \in D(X_i)$ with no support in $D(X_j)$.
3. If $D(X_i)$ changed, add all arcs $(X_k, X_i)$ for neighbors $X_k \neq X_j$ back to the queue.
4. Repeat until queue is empty or any domain becomes empty.

$$\text{Revise}(X_i, X_j): \forall v \in D(X_i), \exists w \in D(X_j) \text{ s.t. } constraint(v, w) = true$$

**Complexity**: O(n² · d³) worst case for general CSPs, but for ternary domains ($d = 3$):

$$O(n^2 \cdot 27) = O(n^2)$$

This makes AC-3 practical for hundreds of ternary variables.

### Backtracking Search with Forward Checking

After AC-3 preprocessing, the backtracking search assigns variables one at a time:

1. **Select** the unassigned variable with **Minimum Remaining Values (MRV)** — the fewest legal values left. This fails fast.
2. **Assign** each value in its domain.
3. **Forward check**: For each neighbor, remove values inconsistent with this assignment. If any neighbor's domain becomes empty, backtrack immediately.
4. **Recurse**. If all variables assigned, return the solution.

**Complexity**: O(3ⁿ) worst case (exhaustive), but forward checking + MRV typically reduces this to O(n · 3^k) where k is the effective branching factor after pruning.

### AllDifferent Constraint

The AllDifferent constraint enforces that all variables in a set take distinct values. With ternary variables, this means:

$$\text{AllDifferent}(x_1, \ldots, x_k): \quad k \leq 3 \text{ is satisfiable}$$

For $k > 3$ variables each with domain {-1, 0, +1}, AllDifferent is **impossible** (pigeonhole principle). The consistency check catches this immediately.

### Ternary N-Queens

The N-Queens problem on a ternary (3×3) board uses coordinates from {-1, 0, +1}:

$$\text{Place } n \text{ queens such that no two share a row, column, or diagonal}$$

For $n = 3$ (standard 3×3 board), there are **0 solutions** — the classic result that 3-Queens has no solution. The solver confirms this via exhaustive search.

For $n = 2$ on a 3×3 board, solutions exist (place queens in different rows/columns).

### Solution Counting

The brute-force counter enumerates all $3^n$ assignments and checks each:

$$\text{count} = |\{a \in \{-1,0,1\}^n : \text{consistent}(a)\}|$$

**Complexity**: O(3ⁿ). For small *n* (≤ 15), this is feasible. For larger *n*, use the backtracking solver which prunes the search tree.

### Complexity Summary

| Algorithm | Time | Space |
|-----------|------|-------|
| `is_consistent` | O(c) | O(1) |
| `ac3` | O(n² · c) | O(n²) |
| `BacktrackingSearch::solve` | O(3ⁿ) worst, O(n · 3^k) typical | O(n) |
| `count_solutions` | O(3ⁿ) | O(n) |
| `find_all_solutions` | O(3ⁿ) | O(s · n) where s = #solutions |

*c = number of constraints, n = number of variables, k = effective branching factor.*

## Quick Start

```rust
use ternary_constraint::{
    TernaryDomain, TernaryCSP, ArcConsistency, BacktrackingSearch,
};

// Create a CSP: three ternary variables, all different
let mut csp = TernaryCSP::new();
csp.add_var("x", TernaryDomain::full());
csp.add_var("y", TernaryDomain::full());
csp.add_var("z", TernaryDomain::full());
csp.add_all_different(vec!["x".into(), "y".into(), "z".into()]);

// Solve with backtracking
let solution = BacktrackingSearch::solve(&csp).unwrap();
assert_ne!(solution["x"], solution["y"]);
assert_ne!(solution["y"], solution["z"]);

// Count all solutions (should be 3! = 6 permutations of {-1, 0, +1})
assert_eq!(csp.count_solutions(), 6);

// Arc consistency
let mut csp2 = TernaryCSP::new();
csp2.add_var("a", TernaryDomain::full());
csp2.add_var("b", TernaryDomain::full());
csp2.add_binary_constraint("a", "b", |a, b| a != b);
assert!(ArcConsistency::ac3(&mut csp2)); // domains remain non-empty
```

## API

### `TernaryDomain`

| Method | Description |
|--------|-------------|
| `full()` | Domain = {-1, 0, +1} |
| `single(v)` | Domain = {v} |
| `from_vec(vals)` | Custom subset (filters out-of-range) |
| `is_empty()` / `is_singleton()` / `size()` | Cardinality queries |
| `contains(v)` / `remove(v)` | Membership and modification |
| `intersect(other)` / `union(other)` | Set operations |

### `TernaryCSP`

| Method | Description |
|--------|-------------|
| `add_var(name, domain)` | Register a variable |
| `add_binary_constraint(v1, v2, fn)` | Binary predicate constraint |
| `add_unary_constraint(v, fn)` | Unary predicate constraint |
| `add_all_different(vars)` | AllDifferent over variable set |
| `is_consistent(&assignment)` | Check assignment validity |
| `neighbors(var)` | Variables constrained with `var` |
| `find_solution()` | Find one solution (brute force) |
| `find_all_solutions()` | Enumerate all solutions |
| `count_solutions()` | Count solutions |

### `ArcConsistency`

| Method | Description |
|--------|-------------|
| `ac3(&mut TernaryCSP) -> bool` | Enforce arc consistency; false if unsat |

### `BacktrackingSearch`

| Method | Description |
|--------|-------------|
| `solve(&TernaryCSP) -> Option<HashMap<String, i8>>` | MRV + forward checking |

### `ternary_n_queens()`

Returns solutions to the 3×3 N-Queens problem (0 solutions for n=3).

## Architecture Notes

The CSP representation uses **function pointers** (`fn(i8, i8) -> bool`) for constraints rather than closures or trait objects. This keeps the structures `Clone + Debug` without allocation, but limits constraints to stateless predicates. For stateful constraints (e.g., "sum to S"), wrap in an outer struct that fixes S at construction time.

AC-3 and backtracking are deliberately separated: AC-3 runs in O(n²) for ternary domains and is used as a **preprocessing** step. Running AC-3 before backtracking can eliminate entire branches of the search tree, often reducing solve time by orders of magnitude.

The brute-force solver (`find_solution`, `count_solutions`) uses early termination: if a partial assignment violates a constraint, it backtracks immediately rather than completing the assignment. This is effectively "backtracking without forward checking or MRV" — simpler but slower than `BacktrackingSearch::solve`.

## References

- **Mackworth, A. K.** (1977). "Consistency in Networks of Relations." *Artificial Intelligence*, 8(1), 99–118. — Original AC-3 algorithm.
- **Dechter, R.** (2003). *Constraint Processing*. — Comprehensive CSP textbook: AC-3, forward checking, MRV, backtracking.
- **Russell, S., & Norvig, P.** (2020). *AI: A Modern Approach* (4th ed.), §6.3 — Constraint satisfaction problems.
- **Bessière, C., & Régin, J.-C.** (2001). "MAC and Combined Heuristics." *Constraints*, 6(1), 53–85. — Maintaining arc consistency during search.
- **Haralick, R. M., & Elliott, G. L.** (1980). "Increasing Tree Search Efficiency for Constraint Satisfaction Problems." *Artificial Intelligence*, 14, 263–313. — Forward checking.

## License

MIT

# ternary-constraint

Constraint satisfaction and propagation for ternary variables — AC-3 arc consistency, backtracking search with forward checking, and ternary N-queens.

## Why This Exists

Constraint satisfaction problems (CSPs) show up everywhere: scheduling, configuration, routing, verification. Most CSP solvers work on large integer or real-valued domains. But when your variables naturally take three values — low/normal/high, reject/neutral/approve, backward/stop/forward — a full-blown SAT solver is overkill.

**ternary-constraint** provides purpose-built CSP algorithms that exploit the small domain size {-1, 0, +1}. With only 3 values per variable, arc consistency prunes aggressively and backtracking terminates fast. The entire search space for *n* ternary variables is just 3^n — small enough to enumerate exactly when needed.

## Core Concepts

| Type | Meaning |
|---|---|
| `TernaryDomain` | A subset of {-1, 0, +1}: full, singleton, or any combination |
| `TernaryCSP` | A constraint satisfaction problem over ternary variables |
| `Constraint` | Binary (two vars), unary (one var), or AllDifferent |
| `ArcConsistency` | AC-3 algorithm for domain pruning |
| `BacktrackingSearch` | MRV heuristic + forward checking solver |

## Quick Start

```toml
# Cargo.toml
[dependencies]
ternary-constraint = "0.1"
```

```rust
use ternary_constraint::*;

fn main() {
    let mut csp = TernaryCSP::new();

    // Three variables, each can be -1, 0, or +1
    csp.add_var("x", TernaryDomain::full());
    csp.add_var("y", TernaryDomain::full());
    csp.add_var("z", TernaryDomain::full());

    // All must be different — exactly 6 solutions (3! permutations)
    csp.add_all_different(vec!["x".into(), "y".into(), "z".into()]);

    // Count and find solutions
    println!("Solutions: {}", csp.count_solutions()); // 6

    let sol = csp.find_solution().unwrap();
    println!("x={}, y={}, z={}", sol["x"], sol["y"], sol["z"]);
}
```

### With Constraint Propagation

```rust
use ternary_constraint::*;

fn main() {
    let mut csp = TernaryCSP::new();
    csp.add_var("a", TernaryDomain::full());
    csp.add_var("b", TernaryDomain::full());
    csp.add_binary_constraint("a", "b", |a, b| a != b);
    csp.add_unary_constraint("a", |v| v > 0); // a must be +1

    // AC-3 prunes domains
    ArcConsistency::ac3(&mut csp);

    // Backtracking with forward checking
    let sol = BacktrackingSearch::solve(&csp);
    println!("{:?}", sol); // a=1, b∈{-1,0}
}
```

## API Overview

### TernaryDomain
- `full()`, `single(v)`, `from_vec(vals)` — constructors
- `is_empty()`, `is_singleton()`, `size()` — inspection
- `contains(v)`, `values()` — membership
- `remove(v)`, `intersect(other)`, `union(other)` — set operations

### TernaryCSP
- `add_var(name, domain)` — declare a variable
- `add_binary_constraint(var1, var2, check_fn)` — relate two variables
- `add_unary_constraint(var, check_fn)` — constrain one variable
- `add_all_different(vars)` — all variables must have distinct values
- `is_consistent(assignment) → bool` — check an assignment
- `find_solution()`, `find_all_solutions()`, `count_solutions()` — brute force
- `neighbors(var)` — variables connected by constraints

### ArcConsistency (AC-3)
- `ac3(&mut csp) → bool` — prune domains; returns false if unsatisfiable

### BacktrackingSearch
- `solve(&csp) → Option<HashMap<String, i8>>` — MRV + forward checking

### Ternary N-Queens
- `ternary_n_queens()` — solve n-queens on a 3×3 ternary board

## How It Works

**AC-3** maintains a queue of arcs (ordered variable pairs connected by binary constraints). For each arc (Xi, Xj), it removes values from Xi's domain that have no supporting value in Xj. If Xi's domain shrinks, all arcs into Xi are re-enqueued. The algorithm terminates when the queue is empty or any domain becomes empty (proving unsatisfiability).

**Backtracking search** selects the unassigned variable with the fewest remaining values (MRV heuristic), tries each domain value, and forward-checks: for each neighbor, it removes values inconsistent with the current assignment. If any neighbor's domain empties, it backtracks immediately. With ternary domains, this pruning is extremely effective.

**All-solutions enumeration** uses recursive depth-first search with early pruning. Since the total space is 3^n, problems up to ~15 variables are tractable for exhaustive enumeration.

## Use Cases

- **Configuration management** — assign ternary settings (disable/auto/enable) to components with mutual exclusion and dependency constraints
- **Scheduling** — ternary shift assignments (off/break/on) with coverage constraints and conflict detection
- **Verification** — check whether a ternary state space has any reachable configuration violating safety constraints

## Ecosystem

Part of the **SuperInstance** ternary computing ecosystem:

- [`ternary`](https://crates.io/crates/ternary) — core trit types and balanced ternary arithmetic
- [`ternary-constraint`](https://crates.io/crates/ternary-constraint) — this crate
- [`ternary-circuit`](https://crates.io/crates/ternary-circuit) — ternary logic gates and circuits
- [`ternary-game-theory`](https://crates.io/crates/ternary-game-theory) — ternary game theory
- [`ternary-control`](https://crates.io/crates/ternary-control) — ternary control theory

## Known Limitations

- **AC-3 only processes binary constraints**: Unary constraints (`Constraint::Unary`) and `AllDifferent` constraints are only applied during backtracking search — the `ac3()` function ignores them entirely. This means AC-3 alone may leave domains larger than they could be.
- **No forward checking**: During backtracking, the solver doesn't prune neighbor domains after each assignment, only backtracks on domain exhaustion. This can be slower than forward-checking solvers.
- **Small domain size**: With only 3 possible values per variable, constraint propagation has limited pruning power. Most real constraint satisfaction happens at larger domain sizes.
- **Dense constraint storage**: All constraints are stored in a flat `Vec` with linear scan for arc lookup — O(n) per constraint check rather than O(1) indexed access.

## License

MIT

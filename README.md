# Ternary Constraint — Constraint Satisfaction and Propagation for Ternary Variables

**Ternary Constraint** implements a constraint satisfaction problem (CSP) solver for variables with ternary domains {-1, 0, +1}. It provides arc-consistency propagation, backtracking search, and support for both binary and unary constraints. The ternary domain is small (|D| = 3), making propagation extremely efficient — each variable can lose at most 2 values before being forced.

## Why It Matters

CSPs are the backbone of scheduling, configuration, and resource allocation problems. When variables have ternary domains, the search space is 3ⁿ for n variables — exponentially large but with a smaller base than boolean CSPs (2ⁿ). The key advantage: ternary domains naturally model three-way decisions (accept/reject/abstain, increase/hold/decrease, active/idle/standby). This crate enables fleet resource scheduling where each allocation decision is ternary: assign (+1), do not assign (-1), or defer (0). Arc-consistency propagation prunes the search space aggressively, often solving problems without any backtracking.

## How It Works

### TernaryDomain

Each variable's domain is a subset of {-1, 0, +1}. The `TernaryDomain` type supports `full()` (all three values), `single(v)` (one value), `from_vec()` (arbitrary subset), and set operations: `intersect()`, `union()`, `remove()`. Domain operations are O(1) since the maximum size is 3.

### Constraint Propagation

The solver enforces **arc consistency (AC-3)**: for each constraint between variables X and Y, every value in X's domain must have at least one supporting value in Y's domain. The AC-3 algorithm uses a queue of arcs to revise:

```
AC-3(csp):
  queue ← all arcs
  while queue not empty:
    (Xi, Xj) ← queue.pop()
    if revise(csp, Xi, Xj):
      if domain(Xi) is empty: return FAILURE
      for each Xk neighbor of Xi (except Xj):
        queue.push((Xk, Xi))
  return SUCCESS
```

Each `revise()` call is O(d²) where d = |D| ≤ 3, so O(9) = O(1). The total AC-3 runtime is O(e × d³) where e is the number of constraints — O(27e) = O(e) for ternary.

### Backtracking Search

When arc consistency alone cannot solve the CSP (domains are non-empty but not singletons), the solver selects the most-constrained variable (minimum remaining values heuristic), assigns a value, propagates, and recurses. Worst-case O(3ⁿ), but with good propagation, typical performance is near-linear.

## Quick Start

```rust
use ternary_constraint::{TernaryDomain, TernaryCSP};

let mut csp = TernaryCSP::new();
csp.add_variable("x", TernaryDomain::full());
csp.add_variable("y", TernaryDomain::full());
csp.add_binary_constraint("x", "y", |a, b| a != b); // x ≠ y

let solution = csp.solve().expect("solution exists");
assert_ne!(solution["x"], solution["y"]);
```

```bash
cargo add ternary-constraint
```

## API

| Type / Function | Description |
|---|---|
| `TernaryDomain` | Subset of {-1, 0, +1}: `full()`, `single(v)`, `intersect()`, `union()` |
| `BinaryConstraint` | `fn(i8, i8) -> bool` — compatibility predicate |
| `TernaryCSP` | Variables + constraints with `solve()`, `add_variable()`, `add_binary_constraint()` |

## Architecture Notes

Constraint satisfaction is used in **SuperInstance** for fleet resource allocation: which GPU nodes get which ternary workloads. The γ + η = C conservation law is itself a constraint: total compute allocation γ plus idle overhead η must equal fleet capacity C. See [Architecture](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md).

## References

- Mackworth, Alan K. "Consistency in Networks of Relations," *Artificial Intelligence*, 8(1), 1977 — arc consistency (AC-3).
- Russell, Stuart & Norvig, Peter. *Artificial Intelligence: A Modern Approach*, 4th ed., 2020 — CSP backtracking search.
- Dechter, Rina. *Constraint Processing*, Morgan Kaufmann, 2003 — comprehensive CSP theory.

## License

MIT

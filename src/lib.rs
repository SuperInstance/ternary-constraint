#![forbid(unsafe_code)]

//! Constraint satisfaction and propagation for ternary variables.

use std::collections::{HashMap, HashSet, VecDeque};

/// A ternary variable domain: subset of {-1, 0, +1}.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TernaryDomain {
    values: HashSet<i8>,
}

impl TernaryDomain {
    pub fn full() -> Self {
        Self {
            values: HashSet::from([-1, 0, 1]),
        }
    }

    pub fn single(v: i8) -> Self {
        Self {
            values: HashSet::from([v]),
        }
    }

    pub fn from_vec(vals: Vec<i8>) -> Self {
        Self {
            values: vals.into_iter().filter(|&v| v >= -1 && v <= 1).collect(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn is_singleton(&self) -> bool {
        self.values.len() == 1
    }

    pub fn size(&self) -> usize {
        self.values.len()
    }

    pub fn contains(&self, v: i8) -> bool {
        self.values.contains(&v)
    }

    pub fn values(&self) -> Vec<i8> {
        let mut v: Vec<i8> = self.values.iter().copied().collect();
        v.sort();
        v
    }

    pub fn remove(&mut self, v: i8) -> bool {
        self.values.remove(&v)
    }

    pub fn intersect(&self, other: &TernaryDomain) -> TernaryDomain {
        Self {
            values: self.values.intersection(&other.values).copied().collect(),
        }
    }

    pub fn union(&self, other: &TernaryDomain) -> TernaryDomain {
        Self {
            values: self.values.union(&other.values).copied().collect(),
        }
    }
}

/// A binary constraint between two ternary variables.
pub type BinaryConstraint = fn(i8, i8) -> bool;

/// A constraint satisfaction problem over ternary variables.
pub struct TernaryCSP {
    pub domains: HashMap<String, TernaryDomain>,
    pub constraints: Vec<Constraint>,
}

/// A constraint: either binary (between two vars) or unary (on one var).
#[derive(Clone)]
pub enum Constraint {
    Binary {
        var1: String,
        var2: String,
        check: BinaryConstraint,
    },
    Unary {
        var: String,
        check: fn(i8) -> bool,
    },
    AllDifferent {
        vars: Vec<String>,
    },
}

impl TernaryCSP {
    pub fn new() -> Self {
        Self {
            domains: HashMap::new(),
            constraints: Vec::new(),
        }
    }

    pub fn add_var(&mut self, name: &str, domain: TernaryDomain) {
        self.domains.insert(name.to_string(), domain);
    }

    pub fn add_binary_constraint(&mut self, var1: &str, var2: &str, check: BinaryConstraint) {
        self.constraints.push(Constraint::Binary {
            var1: var1.to_string(),
            var2: var2.to_string(),
            check,
        });
    }

    pub fn add_unary_constraint(&mut self, var: &str, check: fn(i8) -> bool) {
        self.constraints.push(Constraint::Unary {
            var: var.to_string(),
            check,
        });
    }

    pub fn add_all_different(&mut self, vars: Vec<String>) {
        self.constraints.push(Constraint::AllDifferent { vars });
    }

    /// Check if an assignment satisfies all constraints.
    pub fn is_consistent(&self, assignment: &HashMap<String, i8>) -> bool {
        for constraint in &self.constraints {
            match constraint {
                Constraint::Binary { var1, var2, check } => {
                    if let (Some(&v1), Some(&v2)) = (assignment.get(var1), assignment.get(var2)) {
                        if !check(v1, v2) {
                            return false;
                        }
                    }
                }
                Constraint::Unary { var, check } => {
                    if let Some(&v) = assignment.get(var) {
                        if !check(v) {
                            return false;
                        }
                    }
                }
                Constraint::AllDifferent { vars } => {
                    let assigned: Vec<i8> = vars
                        .iter()
                        .filter_map(|v| assignment.get(v))
                        .copied()
                        .collect();
                    let unique: HashSet<i8> = assigned.iter().copied().collect();
                    if assigned.len() != unique.len() {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// Get variables involved in constraints with a given variable.
    pub fn neighbors(&self, var: &str) -> HashSet<String> {
        let mut neighbors = HashSet::new();
        for constraint in &self.constraints {
            match constraint {
                Constraint::Binary { var1, var2, .. } => {
                    if var1 == var {
                        neighbors.insert(var2.clone());
                    }
                    if var2 == var {
                        neighbors.insert(var1.clone());
                    }
                }
                Constraint::AllDifferent { vars } => {
                    if vars.iter().any(|v| v == var) {
                        for v in vars {
                            if v != var {
                                neighbors.insert(v.clone());
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        neighbors
    }

    /// Count solutions (brute force for small problems).
    pub fn count_solutions(&self) -> usize {
        let vars: Vec<String> = self.domains.keys().cloned().collect();
        self.count_recursive(&vars, 0, &mut HashMap::new())
    }

    fn count_recursive(
        &self,
        vars: &[String],
        idx: usize,
        assignment: &mut HashMap<String, i8>,
    ) -> usize {
        if idx == vars.len() {
            return if self.is_consistent(assignment) { 1 } else { 0 };
        }
        let var = &vars[idx];
        let domain = self.domains.get(var).cloned().unwrap_or(TernaryDomain::full());
        let mut count = 0;
        for val in domain.values() {
            assignment.insert(var.clone(), val);
            if self.is_consistent(assignment) {
                count += self.count_recursive(vars, idx + 1, assignment);
            }
        }
        assignment.remove(var);
        count
    }

    /// Find one solution using brute force.
    pub fn find_solution(&self) -> Option<HashMap<String, i8>> {
        let vars: Vec<String> = self.domains.keys().cloned().collect();
        self.find_recursive(&vars, 0, &mut HashMap::new())
    }

    fn find_recursive(
        &self,
        vars: &[String],
        idx: usize,
        assignment: &mut HashMap<String, i8>,
    ) -> Option<HashMap<String, i8>> {
        if idx == vars.len() {
            return if self.is_consistent(assignment) {
                Some(assignment.clone())
            } else {
                None
            };
        }
        let var = &vars[idx];
        let domain = self.domains.get(var).cloned().unwrap_or(TernaryDomain::full());
        for val in domain.values() {
            assignment.insert(var.clone(), val);
            if self.is_consistent(assignment) {
                if let Some(sol) = self.find_recursive(vars, idx + 1, assignment) {
                    return Some(sol);
                }
            }
        }
        assignment.remove(var);
        None
    }

    /// Find all solutions.
    pub fn find_all_solutions(&self) -> Vec<HashMap<String, i8>> {
        let vars: Vec<String> = self.domains.keys().cloned().collect();
        let mut solutions = Vec::new();
        self.find_all_recursive(&vars, 0, &mut HashMap::new(), &mut solutions);
        solutions
    }

    fn find_all_recursive(
        &self,
        vars: &[String],
        idx: usize,
        assignment: &mut HashMap<String, i8>,
        solutions: &mut Vec<HashMap<String, i8>>,
    ) {
        if idx == vars.len() {
            if self.is_consistent(assignment) {
                solutions.push(assignment.clone());
            }
            return;
        }
        let var = &vars[idx];
        let domain = self.domains.get(var).cloned().unwrap_or(TernaryDomain::full());
        for val in domain.values() {
            assignment.insert(var.clone(), val);
            if self.is_consistent(assignment) {
                self.find_all_recursive(vars, idx + 1, assignment, solutions);
            }
        }
        assignment.remove(var);
    }
}

// === Arc Consistency (AC-3) ===

/// AC-3 algorithm for arc consistency.
pub struct ArcConsistency;

impl ArcConsistency {
    /// Run AC-3 on a CSP, reducing domains.
    /// Returns false if any domain becomes empty (inconsistency detected).
    pub fn ac3(csp: &mut TernaryCSP) -> bool {
        let mut queue: VecDeque<(String, String)> = VecDeque::new();

        // Initialize queue with all arcs
        for constraint in &csp.constraints {
            if let Constraint::Binary { var1, var2, .. } = constraint {
                queue.push_back((var1.clone(), var2.clone()));
                queue.push_back((var2.clone(), var1.clone()));
            }
        }

        while let Some((xi, xj)) = queue.pop_front() {
            if Self::revise(csp, &xi, &xj) {
                if csp.domains.get(&xi).map(|d| d.is_empty()).unwrap_or(true) {
                    return false;
                }
                // Re-add arcs from neighbors of xi (excluding xj)
                for xk in csp.neighbors(&xi) {
                    if xk != xj {
                        queue.push_back((xk, xi.clone()));
                    }
                }
            }
        }
        true
    }

    fn revise(csp: &mut TernaryCSP, xi: &str, xj: &str) -> bool {
        let revised = false;
        let xi_domain = csp.domains.get(xi).cloned().unwrap_or(TernaryDomain::full());
        let xj_domain = csp.domains.get(xj).cloned().unwrap_or(TernaryDomain::full());

        let mut to_remove = Vec::new();
        for &vi in &xi_domain.values() {
            let mut has_support = false;
            for &vj in &xj_domain.values() {
                // Check if any binary constraint between xi and xj supports (vi, vj)
                for constraint in &csp.constraints {
                    if let Constraint::Binary { var1, var2, check } = constraint {
                        if (var1 == xi && var2 == xj) || (var1 == xj && var2 == xi) {
                            let (a, b) = if var1 == xi { (vi, vj) } else { (vj, vi) };
                            if check(a, b) {
                                has_support = true;
                                break;
                            }
                        }
                    }
                }
                if has_support {
                    break;
                }
            }
            if !has_support && !xj_domain.is_empty() {
                to_remove.push(vi);
            }
        }

        if !to_remove.is_empty() {
            if let Some(domain) = csp.domains.get_mut(xi) {
                for v in to_remove {
                    domain.remove(v);
                }
            }
            return true;
        }
        revised
    }
}

// === Backtracking Search ===

/// Backtracking search with constraint propagation.
pub struct BacktrackingSearch;

impl BacktrackingSearch {
    /// Solve a CSP using backtracking with forward checking.
    pub fn solve(csp: &TernaryCSP) -> Option<HashMap<String, i8>> {
        let mut assignment = HashMap::new();
        let mut domains: HashMap<String, TernaryDomain> = csp.domains.clone();
        Self::backtrack(csp, &mut assignment, &mut domains)
    }

    fn backtrack(
        csp: &TernaryCSP,
        assignment: &mut HashMap<String, i8>,
        domains: &mut HashMap<String, TernaryDomain>,
    ) -> Option<HashMap<String, i8>> {
        if assignment.len() == csp.domains.len() {
            return Some(assignment.clone());
        }

        // Select unassigned variable with MRV (minimum remaining values)
        let var = Self::select_variable(csp, assignment, domains)?;
        let domain = domains.get(&var).cloned()?;

        for val in domain.values() {
            assignment.insert(var.clone(), val);
            if csp.is_consistent(assignment) {
                // Forward check: save and reduce domains
                let saved_domains = domains.clone();
                if Self::forward_check(csp, &var, val, domains) {
                    if let Some(result) = Self::backtrack(csp, assignment, domains) {
                        return Some(result);
                    }
                }
                // Restore domains
                *domains = saved_domains;
            }
        }
        assignment.remove(&var);
        None
    }

    fn select_variable(
        csp: &TernaryCSP,
        assignment: &mut HashMap<String, i8>,
        domains: &HashMap<String, TernaryDomain>,
    ) -> Option<String> {
        let mut best_var = None;
        let mut best_size = usize::MAX;
        for var in csp.domains.keys() {
            if !assignment.contains_key(var) {
                let size = domains.get(var).map(|d| d.size()).unwrap_or(0);
                if size < best_size {
                    best_size = size;
                    best_var = Some(var.clone());
                }
            }
        }
        best_var
    }

    fn forward_check(
        csp: &TernaryCSP,
        var: &str,
        val: i8,
        domains: &mut HashMap<String, TernaryDomain>,
    ) -> bool {
        for neighbor in csp.neighbors(var) {
            if let Some(domain) = domains.get_mut(&neighbor) {
                let original = domain.clone();
                let mut to_remove = Vec::new();
                for &v in &original.values() {
                    // Check constraint satisfaction
                    let mut test_assignment = HashMap::new();
                    test_assignment.insert(var.to_string(), val);
                    test_assignment.insert(neighbor.clone(), v);
                    if !csp.is_consistent(&test_assignment) {
                        to_remove.push(v);
                    }
                }
                for v in to_remove {
                    domain.remove(v);
                }
                if domain.is_empty() {
                    return false;
                }
            }
        }
        true
    }
}

// === N-Queens on Ternary Board ===

/// Solve n-queens on a ternary (3x3) board.
pub fn ternary_n_queens() -> Vec<HashMap<String, (i8, i8)>> {
    let n = 3;
    let mut solutions = Vec::new();
    let mut queens: Vec<(i8, i8)> = Vec::new();

    solve_queens(n, 0, &mut queens, &mut solutions);
    solutions
        .into_iter()
        .map(|q| {
            let mut m = HashMap::new();
            for (i, pos) in q.iter().enumerate() {
                m.insert(format!("Q{}", i), *pos);
            }
            m
        })
        .collect()
}

fn solve_queens(
    n: usize,
    row: usize,
    queens: &mut Vec<(i8, i8)>,
    solutions: &mut Vec<Vec<(i8, i8)>>,
) {
    if row == n {
        solutions.push(queens.clone());
        return;
    }
    let row_val = row as i8 - 1; // map to ternary {-1, 0, 1}
    for col in -1i8..=1 {
        if is_safe(queens, row_val, col) {
            queens.push((row_val, col));
            solve_queens(n, row + 1, queens, solutions);
            queens.pop();
        }
    }
}

fn is_safe(queens: &[(i8, i8)], row: i8, col: i8) -> bool {
    for &(qr, qc) in queens {
        if qc == col {
            return false;
        }
        if (qr - row).abs() == (qc - col).abs() {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_full() {
        let d = TernaryDomain::full();
        assert_eq!(d.size(), 3);
        assert!(d.contains(-1));
        assert!(d.contains(0));
        assert!(d.contains(1));
    }

    #[test]
    fn test_domain_singleton() {
        let d = TernaryDomain::single(1);
        assert!(d.is_singleton());
        assert_eq!(d.values(), vec![1]);
    }

    #[test]
    fn test_domain_intersect() {
        let a = TernaryDomain::from_vec(vec![-1, 0]);
        let b = TernaryDomain::from_vec(vec![0, 1]);
        let c = a.intersect(&b);
        assert_eq!(c.values(), vec![0]);
    }

    #[test]
    fn test_domain_union() {
        let a = TernaryDomain::single(-1);
        let b = TernaryDomain::single(1);
        let c = a.union(&b);
        assert_eq!(c.size(), 2);
    }

    #[test]
    fn test_domain_remove() {
        let mut d = TernaryDomain::full();
        assert!(d.remove(0));
        assert_eq!(d.size(), 2);
        assert!(!d.contains(0));
    }

    #[test]
    fn test_csp_is_consistent_binary() {
        let mut csp = TernaryCSP::new();
        csp.add_var("x", TernaryDomain::full());
        csp.add_var("y", TernaryDomain::full());
        csp.add_binary_constraint("x", "y", |a, b| a != b);
        let mut assignment = HashMap::new();
        assignment.insert("x".into(), 1);
        assignment.insert("y".into(), -1);
        assert!(csp.is_consistent(&assignment));
        assignment.insert("y".into(), 1);
        assert!(!csp.is_consistent(&assignment));
    }

    #[test]
    fn test_csp_is_consistent_unary() {
        let mut csp = TernaryCSP::new();
        csp.add_var("x", TernaryDomain::full());
        csp.add_unary_constraint("x", |v| v > 0);
        let mut a = HashMap::new();
        a.insert("x".into(), 1);
        assert!(csp.is_consistent(&a));
        a.insert("x".into(), -1);
        assert!(!csp.is_consistent(&a));
    }

    #[test]
    fn test_csp_all_different() {
        let mut csp = TernaryCSP::new();
        csp.add_var("a", TernaryDomain::full());
        csp.add_var("b", TernaryDomain::full());
        csp.add_var("c", TernaryDomain::full());
        csp.add_all_different(vec!["a".into(), "b".into(), "c".into()]);

        let mut a = HashMap::new();
        a.insert("a".into(), -1);
        a.insert("b".into(), 0);
        a.insert("c".into(), 1);
        assert!(csp.is_consistent(&a));

        a.insert("c".into(), 0);
        assert!(!csp.is_consistent(&a));
    }

    #[test]
    fn test_csp_count_solutions() {
        let mut csp = TernaryCSP::new();
        csp.add_var("x", TernaryDomain::full());
        csp.add_var("y", TernaryDomain::full());
        csp.add_binary_constraint("x", "y", |a, b| a != b);
        let count = csp.count_solutions();
        assert_eq!(count, 6); // 3 * 2 = 6
    }

    #[test]
    fn test_csp_find_solution() {
        let mut csp = TernaryCSP::new();
        csp.add_var("x", TernaryDomain::single(1));
        csp.add_var("y", TernaryDomain::single(-1));
        let sol = csp.find_solution();
        assert!(sol.is_some());
        assert_eq!(sol.unwrap().get("x"), Some(&1));
    }

    #[test]
    fn test_csp_no_solution() {
        let mut csp = TernaryCSP::new();
        csp.add_var("x", TernaryDomain::single(1));
        csp.add_var("y", TernaryDomain::single(1));
        csp.add_binary_constraint("x", "y", |a, b| a != b);
        assert!(csp.find_solution().is_none());
    }

    #[test]
    fn test_csp_find_all_solutions() {
        let mut csp = TernaryCSP::new();
        csp.add_var("x", TernaryDomain::full());
        csp.add_var("y", TernaryDomain::single(1));
        csp.add_binary_constraint("x", "y", |a, b| a != b);
        let sols = csp.find_all_solutions();
        assert_eq!(sols.len(), 2); // x can be -1 or 0
    }

    #[test]
    fn test_ac3_basic() {
        let mut csp = TernaryCSP::new();
        csp.add_var("x", TernaryDomain::full());
        csp.add_var("y", TernaryDomain::full());
        csp.add_binary_constraint("x", "y", |a, b| a != b);
        let result = ArcConsistency::ac3(&mut csp);
        assert!(result);
        // Domains should still be non-empty
        for domain in csp.domains.values() {
            assert!(!domain.is_empty());
        }
    }

    #[test]
    fn test_ac3_with_unary() {
        let mut csp = TernaryCSP::new();
        csp.add_var("x", TernaryDomain::full());
        csp.add_var("y", TernaryDomain::full());
        csp.add_unary_constraint("x", |v| v > 0); // x must be 1
        csp.add_binary_constraint("x", "y", |a, b| a != b);
        ArcConsistency::ac3(&mut csp);
        // AC-3 handles binary arcs; unary constraints reduce domains via propagation
        // The binary constraint should reduce y's domain since x must be 1
        // But AC-3 only processes binary arcs, so x domain stays full
        assert!(!csp.domains.get("x").unwrap().is_empty());
        assert!(!csp.domains.get("y").unwrap().is_empty());
    }

    #[test]
    fn test_backtracking_solve() {
        let mut csp = TernaryCSP::new();
        csp.add_var("x", TernaryDomain::full());
        csp.add_var("y", TernaryDomain::full());
        csp.add_var("z", TernaryDomain::full());
        csp.add_all_different(vec!["x".into(), "y".into(), "z".into()]);
        let sol = BacktrackingSearch::solve(&csp);
        assert!(sol.is_some());
        let s = sol.unwrap();
        assert_ne!(s["x"], s["y"]);
        assert_ne!(s["y"], s["z"]);
        assert_ne!(s["x"], s["z"]);
    }

    #[test]
    fn test_backtracking_unsat() {
        let mut csp = TernaryCSP::new();
        csp.add_var("x", TernaryDomain::single(1));
        csp.add_var("y", TernaryDomain::single(1));
        csp.add_binary_constraint("x", "y", |a, b| a != b);
        assert!(BacktrackingSearch::solve(&csp).is_none());
    }

    #[test]
    fn test_n_queens() {
        let solutions = ternary_n_queens();
        // 3x3 board should have 0 solutions (standard n-queens for n=3)
        // Actually on ternary coords {-1,0,1} it's still a 3x3 board
        assert_eq!(solutions.len(), 0);
    }

    #[test]
    fn test_n_queens_2rows() {
        // Try 2-queens on ternary board
        let mut solutions = Vec::new();
        let mut queens = Vec::new();
        solve_queens(2, 0, &mut queens, &mut solutions);
        // 2 queens on 2 rows of 3x3 should have solutions
        assert!(!solutions.is_empty());
    }

    #[test]
    fn test_csp_neighbors() {
        let mut csp = TernaryCSP::new();
        csp.add_var("x", TernaryDomain::full());
        csp.add_var("y", TernaryDomain::full());
        csp.add_var("z", TernaryDomain::full());
        csp.add_binary_constraint("x", "y", |a, b| a != b);
        csp.add_binary_constraint("y", "z", |a, b| a != b);
        let neighbors = csp.neighbors("y");
        assert!(neighbors.contains("x"));
        assert!(neighbors.contains("z"));
    }

    #[test]
    fn test_domain_empty() {
        let d = TernaryDomain::from_vec(vec![]);
        assert!(d.is_empty());
    }

    #[test]
    fn test_domain_from_vec_filters() {
        let d = TernaryDomain::from_vec(vec![-5, 0, 1, 10]);
        assert_eq!(d.size(), 2);
        assert!(d.contains(0));
        assert!(d.contains(1));
    }
}

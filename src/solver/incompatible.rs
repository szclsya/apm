use super::pool::PkgPool;
use crate::types::PkgMeta;
use varisat::{Lit, Solver};

pub fn find_incompatible_friendly(pool: &dyn PkgPool, to_install: &[usize]) -> String {
    let incompatible = find_incompatible(pool, to_install);
    let pkgs: Vec<&PkgMeta> =
        incompatible.into_iter().map(|id| pool.get_pkg_by_id(id).unwrap()).collect();

    if pkgs.is_empty() {
        "Unknown reason".to_string()
    } else if pkgs.len() == 1 {
        let pkg = pkgs.get(0).unwrap();
        format!(
            "{}({}) appears to have dependency issues that prevents it from being installed. Please contact your package maintainers.",
            pkg.name,
            console::style(&pkg.version).dim()
        )
    } else {
        let mut res = String::from("The following packages cannot be installed simultaneously: ");
        res.push_str("");
        let mut pkgs = pkgs.into_iter().peekable();
        while let Some(pkg) = pkgs.next() {
            res.push_str(&format!("{}({})", pkg.name, console::style(&pkg.version).dim()));
            if pkgs.peek().is_some() {
                res.push_str(", ");
            }
        }
        res
    }
}

fn find_incompatible(pool: &dyn PkgPool, to_install: &[usize]) -> Vec<usize> {
    // Set up solver
    let mut solver = Solver::new();
    let formula = pool.gen_formula(None);
    solver.add_formula(&formula);

    // Check individual packages first
    let to_install_as_lits: Vec<Lit> =
        to_install.iter().map(|id| Lit::from_dimacs(*id as isize)).collect();
    solver.solve().unwrap();
    solver.assume(&to_install_as_lits);
    solver.solve().unwrap();
    let core: Vec<usize> = match solver.failed_core() {
        Some(pkgids) => pkgids.to_vec().into_iter().map(|lit| lit.to_dimacs() as usize).collect(),
        None => Vec::new(),
    };

    core
}

use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use itertools::Itertools;
use maplit::hashmap;
use maplit::hashset;
// use rand::seq::SliceRandom;

use crate::coords::*;
use crate::Plan;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArbAgent {
    pub coord: Coords,
    pub plan: Vec<Actions>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ArbSetup {
    pub agents: Vec<ArbAgent>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ExtractedGame {
    pub setup: ArbSetup,
    pub index2name: Vec<RobotName>,
}


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RobotResult {
    pub plan: Plan,
    pub cost: Cost,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct ArbSolution {
    /// for each agent, the coords
    pub perm: Vec<usize>,
    pub costs: Costs,
    pub robots: Vec<RobotResult>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ArbResult {
    pub solutions: HashMap<Costs, HashSet<ArbSolution>>,
}

impl ArbResult {
    pub fn pick_one(&self, rng: &mut RNG) -> ArbSolution {
        if self.solutions.len() == 0 {
            panic!("pick_one: no solutions");
        }
        // pick a random solution from self.solutions
        let costs = sample_from_hashmap(&self.solutions, rng);
        let equivalent = &self.solutions[&costs];
        sample_from_hashset(equivalent, rng).clone()
    }
    pub fn remove_redundant(&self, rng: &mut RNG) -> Self {
        let mut solutions: HashMap<Costs, HashSet<ArbSolution>> = Default::default();

        for (c, equivalent) in self.solutions.iter() {
            if equivalent.len() > 1 {
                let one = sample_from_hashset(&equivalent, rng);
                solutions.insert(c.clone(), hashset![one]);
            } else {
                solutions.insert(c.clone(), equivalent.clone());
            }
        }

        Self { solutions }
    }
}

type RS = (usize, XYCell);
type RSM = HashMap<RS, usize>;

pub fn get_resources_needed(
    t0: usize,
    coord: &Coords,
    action: Actions,
    robot_name: RobotName,
) -> RSM {
    let coord = coord;
    let coord2 = next_coords(coord, action);
    hashmap![(t0, coord.xy) => robot_name, (t0 , coord2.xy)=>robot_name,
    (t0 +1, coord2.xy)=>robot_name]
}

pub fn are_resources_available(resources_committed: &RSM, resources: &RSM) -> bool {
    for (rs, robot) in resources.iter() {
        if resources_committed.contains_key(rs) {
            let other = resources_committed[rs];
            if other != *robot {
                // eprintln!("found conflict at {rs:?} for {robot} already {other}");
                return false;
            }
        }
    }
    true
}

pub fn is_action_feasible(
    resources_committed: &RSM,
    robot_name: RobotName,
    coord: &Coords,
    t0: usize,
    action: Actions,
) -> Option<(Coords, RSM)> {
    // see if we can add these resources
    let coord2 = next_coords(coord, action);

    let new_resources = get_resources_needed(t0, coord, action, robot_name);
    // eprintln!("is_action_feasible for {robot_name} at {coord:?} going to {coord2:?} @ {t0} = new_resources = {:?}", new_resources);
    if !are_resources_available(resources_committed, &new_resources) {
        // eprintln!("is_action_feasible: resources not available");
        None
    } else {
        // add the resources
        let mut resources_committed = resources_committed.clone();

        for (rs, robot_name) in new_resources {
            // assert!(!resources_committed.contains_key(&rs));
            mark_occupied(&mut resources_committed, rs.0, &rs.1, robot_name);
            // resources_committed.insert(rs, robot_name);
        }

        Some((coord2, resources_committed))
    }
}

pub fn assign_actions(
    resources: &RSM,
    robot_name: RobotName,
    coord: &Coords,
    actions_committed: &Vec<Actions>,
    actions_remaining: &Vec<Actions>,
) -> Option<(RSM, Vec<Actions>)> {
    // eprintln!("assign_actions for {robot_name}: actions_committed = {:?}, actions_remaining = {:?}, \
    // resources= {resources:?}",
    //           actions_committed, actions_remaining);
    if actions_remaining.len() == 0 {
        return Some((resources.clone(), actions_committed.clone()));
    }
    let delay = actions_committed.len();
    let action = actions_remaining[0];
    if let Some((coord2, r)) = is_action_feasible(&resources, robot_name, &coord, delay, action) {
        let mut actions_committed = actions_committed.clone();
        actions_committed.push(action);
        let actions_remaining = actions_remaining[1..].to_vec();
        assign_actions(
            &r,
            robot_name,
            &coord2,
            &actions_committed,
            &actions_remaining,
        )
    } else {
        // eprintln!("assign_actions: adding wait for {robot_name} at {coord:?} at delay {delay}",);
        let mut actions_committed = actions_committed.clone();
        actions_committed.push(Actions::Wait);
        let mut resources = resources.clone();


        if occupied_by_someone_else(&resources, delay, &coord.xy, robot_name) {
            None
        } else {
            mark_occupied(&mut resources, delay, &coord.xy, robot_name);

            assign_actions(
                &resources,
                robot_name,
                coord,
                &actions_committed,
                &actions_remaining,
            )
        }
    }
}


pub fn occupied_by_someone_else(
    resources: &RSM,
    t0: usize,
    xy: &XYCell,
    robot_name: RobotName,
) -> bool {
    let key = (t0, *xy);
    if resources.contains_key(&key) {
        let other = resources[&key];
        if other != robot_name {
            return true;
        }
    }
    false
}

pub fn mark_occupied(resources: &mut RSM, t0: usize, xy: &XYCell, robot_name: RobotName) {
    let key = (t0, *xy);
    if resources.contains_key(&key) {
        let other = resources[&key];
        if other != robot_name {
            panic!("mark_occupied: found conflict at {key:?} for {robot_name} already {other}");
        }
    } else {
        resources.insert(key, robot_name);
    }
}

pub fn assign(s: &ArbSetup, order: &Vec<usize>) -> Option<(RSM, ArbSolution)> {
    let mut resources: RSM = Default::default();
    for (a, agent) in s.agents.iter().enumerate() {
        mark_occupied(&mut resources, 0, &agent.coord.xy, a);
    }

    let mut agents_results: Vec<RobotResult> = Default::default();
    for _ in order {
        agents_results.push(RobotResult {
            plan: Plan::default(),
            cost: 0,
        });
    }
    for i in order {
        let agent = &s.agents[*i];

        let x = assign_actions(&resources, *i, &agent.coord, &Vec::new(), &agent.plan);
        match x {
            None => return None,
            Some((r, acts)) => {
                resources = r;
                agents_results[*i].plan = acts.clone();
                // count the number of Wait actions
                agents_results[*i].cost = acts.iter().filter(|a| **a == Actions::Wait).count();
            }
        };
    }
    let costs = agents_results.iter().map(|r| r.cost).collect_vec();

    let a = ArbSolution {
        perm: order.clone(),
        costs,
        robots: agents_results.clone(),
    };
    Some((resources, a))
}

pub fn le(a: &Vec<usize>, b: &Vec<usize>) -> bool {
    leq(a, b) && a != b
}

pub fn leq(a: &Vec<usize>, b: &Vec<usize>) -> bool {
    let n = a.len();
    if n != b.len() {
        panic!("leq: lengths differ");
    }
    for i in 0..n {
        if !(a[i] <= b[i]) {
            return false;
        }
    }
    true
}

// pub fn factorial(n: usize) -> usize {
//     let mut f = 1;
//     for i in 1..n {
//         f *= i;
//     }
//     f
// }

pub fn max_n_where_permutation_less_than(max: usize) -> usize {
    let mut n = 1;
    let mut f = 1;
    while f < max {
        n += 1;
        f *= n;
    }
    n - 1
}

/// Returns the permutations of a set of n elements (0..n)
/// with a maximum of max elements. Give 0 for max to get all permutations.
pub fn get_permutations(n: usize, max: usize) -> Vec<Vec<usize>> {
    let n0 = if max == 0 { n } else {
        max_n_where_permutation_less_than(max)
    };
    // eprintln!("get_permutations({n}, {max}) -> n0 = {n0}", n = n, max = max, n0 = n0);
    if n <= n0 {
        let mut res = Vec::new();

        let perms = (0..n).permutations(n);
        for p in perms {
            res.push(p);
        }
        res
    } else {
        get_permutations(n0, max)
            .iter()
            .map(|p| {
                let mut v = p.clone();
                for i in n0..n {
                    v.push(i);
                }
                v
            })
            .collect_vec()
    }
}


pub fn find_feasible_plans(s: &ArbSetup, max: usize) -> ArbResult {
    let n = s.agents.len();


    let perms = get_permutations(n, max);


    let mut solutions: HashMap<Costs, HashSet<ArbSolution>> = HashMap::new();
    for perm in perms {
        match &assign(s, &perm) {
            None => {
                // eprintln!("{perm:?} -> FAIL");
                continue;
            }
            Some((_, solution)) => {
                // let mut costs = Vec::with_capacity(n).fill(0);
                // let costs = solution.robots.iter().map(|r| r.cost).collect_vec();
                // let plans = solution.iter().map(|r| r.actions.clone()).collect_vec();

                for c in solutions.keys() {
                    if le(c, &solution.costs) {
                        // dominated
                        continue;
                    }
                }
                // eprintln!("{costs:?} is minimal ");

                solutions
                    .entry(solution.costs.clone())
                    .or_default()
                    .insert(solution.clone());
                let to_remove: Vec<Vec<usize>> = solutions
                    .keys()
                    .filter(|c| le(&solution.costs, &c))
                    .map(|x| x.clone())
                    .collect_vec();

                for d in &to_remove {
                    solutions.remove(d);
                }
            }
        }

        // let mut r: Vec<HashMap<RobotName, HashSet<XYCell>>> = Default::default();
        // let max_t = rsm.keys().map(|(t, _)| t).max().unwrap();
        // for t in 0..*max_t {
        //     let mut r_t: HashMap<RobotName, HashSet<XYCell>> = Default::default();
        //     for ((t2, xy), robot_name) in rsm.iter() {
        //         if t2 != &t {
        //             continue;
        //         }
        //         // r[t][robot_name].insert(xy)
        //         r_t.entry(*robot_name).or_default().insert(*xy);
        //     }
        //     eprintln!("t = {t}: {r_t:?}");
        //
        //     r.insert(t, r_t);
        // }
    }


    ArbResult { solutions }
}

const F: Actions = Actions::Forward;
const R: Actions = Actions::TurnRight;


#[cfg(test)]
mod test {
    use crate::*;

    use super::*;

    fn get_E(ord: usize, nsteps: usize) -> ArbAgent {
        ArbAgent {
            coord: Coords::from(XYCell::new(1 + (ord as i16), 0), Orientations::WEST),
            plan: vec![F; nsteps],
        }
    }

    fn get_N(ord: usize, nsteps: usize) -> ArbAgent {
        ArbAgent {
            coord: Coords::from(XYCell::new(-1 - (ord as i16), 1), Orientations::SOUTH),
            plan: vec![F; nsteps],
        }
    }

    fn get_W(ord: usize, nsteps: usize) -> ArbAgent {
        ArbAgent {
            coord: Coords::from(XYCell::new(-2 + (ord as i16), -1), Orientations::EAST),
            plan: vec![F; nsteps],
        }
    }

    fn get_S(ord: usize, nsteps: usize) -> ArbAgent {
        ArbAgent {
            coord: Coords::from(XYCell::new(0, -2 - (ord as i16)), Orientations::NORTH),
            plan: vec![F; nsteps],
        }
    }


    #[test]
    fn test_arb1() {
        let H = 3;
        let E1 = get_E(0, H);
        let N1 = get_N(0, H);
        let W1 = get_W(0, H);
        let S1 = get_S(0, H);

        let agents = vec![E1, N1, W1, S1];
        let setup = ArbSetup { agents };
        let result = find_feasible_plans(&setup, 0);
        eprintln!("result = {result:?}", result = result.solutions.keys());
        let result_min = result.remove_redundant(&mut RNG::default());
        assert_eq!(result.solutions.len(), 6);
        // check that the symmetric case has 6 solutions (4 + 2 criss cross)
    }

    #[test]
    fn test_arb2() {
        let rng = &mut RNG::default();

        let n = 10;
        let H = 3;

        let mut agents = (0..n).map(|i| get_E(i, H)).collect_vec();

        agents.shuffle(rng);

        let setup = ArbSetup { agents };
        let result = find_feasible_plans(&setup, 1000);
        assert_eq!(result.solutions.len(), 1);
        eprintln!("result = {result:?}", result = result.solutions.keys());
    }
}

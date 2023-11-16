use std::collections::{HashMap, HashSet};
use std::collections::hash_map;
use std::hash::Hash;

use itertools::Itertools;
use maplit::hashmap;

use crate::coords::*;

pub struct ArbAgent {
    // pub name: usize,
    pub coord: Coords,
    pub plan: Vec<Actions>,
}

pub struct ArbSetup {
    pub agents: Vec<ArbAgent>,
}

// Import the Itertools trait to get access to its methods


pub struct ArbStep {
    /// for each agent, the coords
    pub coords: Vec<Coords>,
    pub actions: Vec<Option<Actions>>,
}

pub struct ArbResult {
    pub steps: Vec<ArbStep>,
}

type RS = (usize, XYCell);
type RSM = HashMap<RS, usize>;

pub fn get_resources_needed(t0: usize, coord: &Coords, action: Actions, robot_name: RobotName) -> RSM {
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

pub fn is_action_feasible(resources_committed: &RSM, robot_name: RobotName, coord: &Coords, t0: usize, action: Actions) -> Option<(Coords, RSM)> {
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

pub fn assign_actions(resources: &RSM, robot_name: RobotName, coord: &Coords, actions_committed: &Vec<Actions>,
                      actions_remaining: &Vec<Actions>) -> Option<(RSM, Vec<Actions>)> {
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
        assign_actions(&r, robot_name, &coord2, &actions_committed, &actions_remaining)
    } else {
        // eprintln!("assign_actions: adding wait for {robot_name} at {coord:?} at delay {delay}",);
        let mut actions_committed = actions_committed.clone();
        actions_committed.push(Actions::Wait);
        let mut resources = resources.clone();
        //
        // if resources.contains_key(&(delay, coord.xy)) {
        //     let other = resources[&(delay, coord.xy)];
        //     if other != robot_name {
        //         panic!("assign_actions: found conflict at {delay} for {robot_name} already {other}");
        //     }
        // }
        if occupied_by_someone_else(&resources, delay, &coord.xy, robot_name) {
            return None;
        }
        // assert!(!resources.contains_key(&(delay, coord.xy)));
        mark_occupied(&mut resources, delay, &coord.xy, robot_name);
        // resources.insert((delay, coord.xy), robot_name);

        assign_actions(&resources, robot_name, coord, &actions_committed, &actions_remaining)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RobotResult {
    pub actions: Vec<Actions>,
    pub cost: usize,
}

pub fn occupied_by_someone_else(resources: &RSM, t0: usize, xy: &XYCell, robot_name: RobotName) -> bool {
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

pub fn assign(s: &ArbSetup, order: &Vec<usize>) -> Option<(RSM, Vec<RobotResult>)> {
    let mut resources: RSM = Default::default();
    for (a, agent) in s.agents.iter().enumerate() {
        mark_occupied(&mut resources, 0, &agent.coord.xy, a);
    }

    let mut agents_results: Vec<RobotResult> = Default::default();
    for _ in order {
        agents_results.push(RobotResult { actions: Vec::new(), cost: 0 });
    }
    for i in order {
        let agent = &s.agents[*i];

        let x = assign_actions(&resources, *i, &agent.coord, &Vec::new(), &agent.plan);
        match x {
            None => return None,
            Some((r, acts)) => {
                resources = r;
                agents_results[*i].actions = acts.clone();
                // count the number of Wait actions
                agents_results[*i].cost = acts.iter().filter(|a| **a == Actions::Wait).count();
            }
        };
    }
    Some((resources, agents_results))
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

pub fn arbitration(s: &ArbSetup) -> ArbResult {
    let n = s.agents.len();

    let perms = (0..n).permutations(n);

    let mut costs_found: HashMap<Vec<usize>, HashSet<Vec<RobotResult>>> = HashMap::new();

    for perm in perms {
        match assign(s, &perm) {
            None => {
                // eprintln!("{perm:?} -> FAIL");
                continue;
            }
            Some((_rsm, solution)) => {
                // let mut costs = Vec::with_capacity(n).fill(0);
                let costs = solution.iter().map(|r| r.cost).collect_vec();
                // let plans = solution.iter().map(|r| r.actions.clone()).collect_vec();


                for c in costs_found.keys() {
                    if le(c, &costs) {
                        // dominated
                        continue;
                    }
                }
                // eprintln!("{costs:?} is minimal ");

                costs_found.entry(costs.clone()).or_default().insert(solution);
                let to_remove: Vec<Vec<usize>> = costs_found.keys().filter(|c| le(&costs, &c)).map(|x| x.clone()).collect_vec();

                for d in &to_remove {
                    costs_found.remove(d);
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

    for (costs, solutions) in costs_found.iter() {
        for (a, solution) in solutions.iter().enumerate() {
            let plans = solution.iter().map(|r| r.actions.clone()).collect_vec();
            eprintln!("{costs:?} -> #{a} {plans:?}");
        }


        // eprintln!("{costs:?} -> {solutions:?}");
    }


    ArbResult {
        steps: Vec::new(),
    }
}

const F: Actions = Actions::Forward;
const R: Actions = Actions::TurnRight;

#[cfg(test)]
mod test {
    use crate::*;

    use super::*;

    #[test]
    fn test_arb1() {
        //

        let E1 = ArbAgent {
            coord: Coords::from(XYCell::new(1, 0), Orientations::WEST),
            plan: vec![F, F, F],
        };
        let E2 = ArbAgent {
            coord: Coords::from(XYCell::new(2, 0), Orientations::WEST),
            plan: vec![F, F, F],
        };
        let E3 = ArbAgent {
            coord: Coords::from(XYCell::new(3, 0), Orientations::WEST),
            plan: vec![F, F, F],
        };
        let N1 = ArbAgent {
            coord: Coords::from(XYCell::new(-1, 1), Orientations::SOUTH),
            plan: vec![F, F, F],
        };
        let W1 = ArbAgent {
            coord: Coords::from(XYCell::new(-2, -1), Orientations::EAST),
            plan: vec![F, F, F],
        };
        let S1 = ArbAgent {
            coord: Coords::from(XYCell::new(0, -2), Orientations::NORTH),
            plan: vec![F, F, F],
        };
        let agents = vec![
            E1, E2, E3,
            N1,
            W1,
            S1,
            // ArbAgent {
            //     coord: Coords::from(XYCell::new(-1, 1), Orientations::SOUTH),
            //     plan: vec![F, F, F],
            // },
            // ArbAgent {
            //     coord: Coords::from(XYCell::new(-2, -1), Orientations::EAST),
            //     plan: vec![F, F, F],
            // },
            // ArbAgent {
            //     coord: Coords::from(XYCell::new(0, -2), Orientations::NORTH),
            //     plan: vec![F, F, F],
            // },
        ];
        let setup = ArbSetup { agents };

        arbitration(&setup);
        // todo: check that the symmetric case has 6 solutions (4 + 2 criss cross)
    }
}

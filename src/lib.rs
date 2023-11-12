#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod coords;
pub use app::WorldViewApp;
pub use coords::*;
mod blocks;
pub use blocks::*;

// pub fn add(left: usize, right: usize) -> usize {
//     left + right
// }
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
// type AgentName = String;
// type AgentState = f32;
// type Command = String;
// type JointAction = HashMap<AgentName, Command>;
// type JointState = HashMap<AgentName, State>;
//
//
// struct Probability <T> {
//     possibilities: Vec<(f32, T)>,
// }
//
// struct DPGNode<AgentName, Command, AgentState> {
//     pub agent2state: JointState,
//     pub agent2actions: HashMap<AgentName, Vec<Command>>,
//     pub transition: HashMap<
//         JointAction,
//          Probability<HashMap<AgentMap, (AgentName, JointState)>>>,
// }
//
// struct DPG<AgentName, Command, AgentState> {
//     pub nodes: HashMap<JointState, DPGNode>,
// }

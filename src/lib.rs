#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod coords;
pub use app::WorldViewApp;
pub use coords::*;
mod blocks;
pub use blocks::*;
mod efficient_setsampling;
pub use efficient_setsampling::*;
mod arbitration;
pub use arbitration::*;

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

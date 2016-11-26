// extern crate daggy;
// #[macro_use]
// extern crate custom_derive;
// #[macro_use]
// extern crate enum_derive;
//
// pub extern crate palette;

use std::slice;
use std::collections::HashSet;
use std::ops::Deref;

use daggy::{PetGraph, NodeIndex};
use daggy::petgraph::graph;
use daggy::petgraph::Bfs;

use self::dag::PortNumbered;
use self::gate::Gate;

pub use self::dag::{Edge, Port, port};

pub mod gate;
mod dag;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    Added,
    Changed,
    Removed,
}

pub struct Game<T> {
    dag: PortNumbered<Node<T>>
}

pub struct GameView<'a, T: 'a> (&'a Game<T>);

impl<'a, T: 'a> GameView<'a, T> {
    pub fn get(&self, node: NodeIndex) -> Option<(&Gate, &T)> {
        self.0.dag.node_weight(node).map(|n| (&*n.process, &n.data))
    }
}

impl<'a, T: 'a> Deref for GameView<'a, T> {
    type Target = Game<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Game<T> {
    pub fn new() -> Game<T> {
        Game {
            dag: PortNumbered::new(),
        }
    }

    pub fn view<F>(&mut self) -> GameView<T> {
        GameView(&*self)
    }

    pub fn get(&self, node: NodeIndex) -> Option<(&Box<Gate>, &T)> {
        self.dag.node_weight(node).map(|n| (&n.process, &n.data))
    }

    pub fn get_process_mut(&mut self, node: NodeIndex) -> Option<&mut Box<Gate>> {
        self.dag.node_weight_mut(node).map(|n| &mut n.process)
    }

    pub fn get_data_mut(&mut self, node: NodeIndex) -> Option<&mut T> {
        self.dag.node_weight_mut(node).map(|n| &mut n.data)
    }

    pub fn add(&mut self, node: Box<Gate>, data: T) -> NodeIndex {
        self.dag.add_node(Node::new(node, data))
    }

    pub fn remove(&mut self, node: &NodeIndex) -> Option<(Box<Gate>, T)> {
        let children = self.dag.children(*node).map(|n| n.1.node).collect::<Vec<_>>();
        self.dag.remove_outgoing_edges(*node);
        if let Some(n) = self.dag.remove_node(*node) {
            Some((n.process, n.data))
        } else {
            None
        }
    }

    pub fn connect(&mut self, from: Port<u32>, to: Port<u32>) -> bool {
        self.dag.update_edge(from, to).is_ok()
    }

    pub fn disconnect(&mut self, to: Port<u32>) -> Option<Port<u32>> {
        self.dag.remove_edge_to_port(to)
    }

    pub fn graph(&self) -> &PetGraph<Node<T>, dag::Edge, u32> {
        self.dag.graph()
    }

    pub fn iter(&self) -> Iter<T> {
        Iter(self.dag.raw_nodes().iter())
    }

    pub fn iter_connections(&self) -> dag::Edges<u32> {
        self.dag.edges()
    }

    pub fn connections(&self) -> usize {
        self.dag.edge_count()
    }
}

pub struct Node<T> {
    data: T,
    process: Box<Gate>,
}

impl<T> Node<T> {
    fn new(process: Box<Gate>, data: T) -> Node<T> {
        Node {
            data: data,
            process: process,
        }
    }
}

pub struct Iter<'a, T: 'a>(slice::Iter<'a, graph::Node<Node<T>>>);

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = (&'a Gate, &'a T);
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|n| (&*n.weight.process, &n.weight.data))
    }
}

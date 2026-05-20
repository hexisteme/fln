use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Causal node kind — Pearl do-calculus 의 역할 구분.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    Cause,
    Effect,
    Confounder,
    Mediator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalNode {
    pub id: String,
    pub label: String,
    pub kind: NodeKind,
}

/// Edge kind — Direct (X→Y), Confounded (X←Z→Y observed through Z), Backdoor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind {
    Direct,
    Confounded,
    Backdoor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalEdge {
    pub from: String,
    pub to: String,
    pub kind: EdgeKind,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CausalDAG {
    pub nodes: Vec<CausalNode>,
    pub edges: Vec<CausalEdge>,
}

#[derive(Debug, thiserror::Error)]
pub enum CausalError {
    #[error("node id `{0}` already exists")]
    DuplicateNode(String),
    #[error("edge endpoint `{0}` is unknown")]
    UnknownEndpoint(String),
    #[error("edge would introduce a cycle: {0} -> {1}")]
    CycleIntroduced(String, String),
}

impl CausalDAG {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, node: CausalNode) -> Result<(), CausalError> {
        if self.nodes.iter().any(|n| n.id == node.id) {
            return Err(CausalError::DuplicateNode(node.id));
        }
        self.nodes.push(node);
        Ok(())
    }

    pub fn add_edge(&mut self, edge: CausalEdge) -> Result<(), CausalError> {
        if !self.has_node(&edge.from) {
            return Err(CausalError::UnknownEndpoint(edge.from));
        }
        if !self.has_node(&edge.to) {
            return Err(CausalError::UnknownEndpoint(edge.to));
        }
        if self.path_exists(&edge.to, &edge.from) {
            return Err(CausalError::CycleIntroduced(edge.from, edge.to));
        }
        self.edges.push(edge);
        Ok(())
    }

    pub fn has_node(&self, id: &str) -> bool {
        self.nodes.iter().any(|n| n.id == id)
    }

    /// DFS reachability — is there a directed path `from → to`?
    pub fn path_exists(&self, from: &str, to: &str) -> bool {
        let mut adjacency: HashMap<&str, Vec<&str>> = HashMap::new();
        for e in &self.edges {
            adjacency.entry(e.from.as_str()).or_default().push(e.to.as_str());
        }
        let mut stack = vec![from];
        let mut seen = HashSet::new();
        while let Some(cur) = stack.pop() {
            if cur == to {
                return true;
            }
            if !seen.insert(cur) {
                continue;
            }
            if let Some(next) = adjacency.get(cur) {
                stack.extend(next.iter().copied());
            }
        }
        false
    }

    /// Kahn's algorithm — topological order; `None` 시 cycle (add_edge 가 막아 정상 케이스 없음).
    pub fn topological_order(&self) -> Option<Vec<String>> {
        let mut indegree: HashMap<&str, usize> = self.nodes.iter().map(|n| (n.id.as_str(), 0)).collect();
        let mut adjacency: HashMap<&str, Vec<&str>> = HashMap::new();
        for e in &self.edges {
            *indegree.entry(e.to.as_str()).or_insert(0) += 1;
            adjacency.entry(e.from.as_str()).or_default().push(e.to.as_str());
        }
        let mut queue: Vec<&str> = indegree
            .iter()
            .filter_map(|(k, v)| if *v == 0 { Some(*k) } else { None })
            .collect();
        queue.sort();
        let mut order = Vec::with_capacity(self.nodes.len());
        while let Some(cur) = queue.pop() {
            order.push(cur.to_string());
            if let Some(next) = adjacency.get(cur) {
                let mut released = Vec::new();
                for n in next {
                    let entry = indegree.entry(*n).or_insert(0);
                    if *entry > 0 {
                        *entry -= 1;
                    }
                    if *entry == 0 {
                        released.push(*n);
                    }
                }
                released.sort();
                queue.extend(released);
            }
        }
        if order.len() == self.nodes.len() { Some(order) } else { None }
    }

    /// JSON Schema (draft-2020-12) 카테고리만 — IETF draft 의 baseline.
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(id: &str, kind: NodeKind) -> CausalNode {
        CausalNode { id: id.into(), label: id.into(), kind }
    }
    fn edge(from: &str, to: &str) -> CausalEdge {
        CausalEdge { from: from.into(), to: to.into(), kind: EdgeKind::Direct }
    }

    #[test]
    fn add_node_rejects_duplicate() {
        let mut g = CausalDAG::new();
        g.add_node(node("X", NodeKind::Cause)).unwrap();
        let err = g.add_node(node("X", NodeKind::Effect)).unwrap_err();
        matches!(err, CausalError::DuplicateNode(_));
    }

    #[test]
    fn add_edge_rejects_unknown_endpoint() {
        let mut g = CausalDAG::new();
        g.add_node(node("X", NodeKind::Cause)).unwrap();
        let err = g.add_edge(edge("X", "Y")).unwrap_err();
        matches!(err, CausalError::UnknownEndpoint(_));
    }

    #[test]
    fn add_edge_rejects_cycle() {
        let mut g = CausalDAG::new();
        g.add_node(node("X", NodeKind::Cause)).unwrap();
        g.add_node(node("Y", NodeKind::Effect)).unwrap();
        g.add_edge(edge("X", "Y")).unwrap();
        let err = g.add_edge(edge("Y", "X")).unwrap_err();
        matches!(err, CausalError::CycleIntroduced(_, _));
    }

    #[test]
    fn topological_order_respects_dependencies() {
        let mut g = CausalDAG::new();
        g.add_node(node("A", NodeKind::Cause)).unwrap();
        g.add_node(node("B", NodeKind::Mediator)).unwrap();
        g.add_node(node("C", NodeKind::Effect)).unwrap();
        g.add_edge(edge("A", "B")).unwrap();
        g.add_edge(edge("B", "C")).unwrap();
        let order = g.topological_order().unwrap();
        let pos = |id: &str| order.iter().position(|x| x == id).unwrap();
        assert!(pos("A") < pos("B"));
        assert!(pos("B") < pos("C"));
    }

    #[test]
    fn json_roundtrip() {
        let mut g = CausalDAG::new();
        g.add_node(node("VIX", NodeKind::Confounder)).unwrap();
        g.add_node(node("BTC", NodeKind::Effect)).unwrap();
        g.add_edge(edge("VIX", "BTC")).unwrap();
        let s = g.to_json().unwrap();
        let parsed: CausalDAG = serde_json::from_str(&s).unwrap();
        assert_eq!(parsed.nodes.len(), 2);
        assert_eq!(parsed.edges.len(), 1);
    }
}

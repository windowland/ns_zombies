use crate::happenings::Event;
use educe::Educe;
use regex::Regex;
#[derive(Eq, PartialEq, Clone, PartialOrd, Ord, Debug, Copy, IsVariant)]
pub enum EventType<'a> {
  Zombie {
    level: &'a str,
    converted: bool,
    affected: usize,
  },
  Cure {
    level: usize,
    restored: bool,
    affected: usize,
  },
  Kill {
    level: usize,
    affected: usize,
  },
  Move {
    nation: &'a str,
  },
}
impl EventType<'_> {
  pub fn is_attack(&self) -> bool {
    !self.is_move()
  }
  pub fn stats_incoming(self) -> EventStats {
    match self {
      EventType::Cure { affected, .. } => EventStats {
        cured_by_others: affected,
        hit_by_missiles: 1,
        ..Default::default()
      },
      EventType::Kill { affected, .. } => EventStats {
        killed_by_others: affected,
        hit_by_tzes: 1,
        ..Default::default()
      },
      EventType::Zombie { affected, .. } => EventStats {
        zombified_by_others: affected,
        hit_by_hordes: 1,
        ..Default::default()
      },
      _ => Default::default(),
    }
  }
  pub fn stats_outgoing(self) -> EventStats {
    match self {
      EventType::Cure { affected, .. } => EventStats {
        others_cured: affected,
        missiles_used: 1,
        min_time: 20,
        ..Default::default()
      },
      EventType::Kill { affected, .. } => EventStats {
        others_killed: affected,
        tzes_used: 1,
        min_time: 20,
        ..Default::default()
      },
      EventType::Zombie { affected, .. } => EventStats {
        others_zombified: affected,
        hordes_used: 1,
        min_time: 20,
        ..Default::default()
      },
      _ => Default::default(),
    }
  }
}
#[derive(Educe, Eq, Debug, Clone, Copy)]
#[educe(PartialEq, PartialOrd, Ord)]
pub struct ZEvent<'a> {
  #[educe(PartialEq(ignore), PartialOrd(ignore), Ord(ignore))]
  pub id: u64,
  pub timestamp: u64,
  pub from: &'a str,
  pub to: &'a str,
  pub event: EventType<'a>,
}
lazy_static::lazy_static! {
  static ref CURE:Regex = Regex::new(
    "^@@(?P<to>[a-z0-9_\\-]*)@@ was struck by a Mk (?P<level>[IV]{1,3}) \\([a-zA-Z]*\\) \
    Cure Missile from @@(?P<from>[a-z0-9_\\-]*)@@, curing (?P<affected>[\\d,]*) million infected(\\.)| (p<restore>\
    and restoring to a zombie researcher!)$"
  ).unwrap();
  static ref ZOMBIE:Regex = Regex::new("^@@(?P<to>[a-z0-9_\\-]*)@@ was ravaged by a Zombie \
    (?P<level>[a-zA-Z]*) Horde from @@(?P<from>[a-z0-9_\\-]*)@@, infecting (?P<affected>[\\d,]*) \
    million survivors((\\.)|(?P<convert> and converting to a zombie exporter! Oh no!))$"
  ).unwrap();
  static ref KILL:Regex = Regex::new("^@@(?P<to>[a-z0-9_\\-]*)@@ was cleansed by a Level (?P<level>[1-5]) \
  [a-zA-Z]* Tactical Zombie Elimination Squad from @@(?P<from>[a-z0-9_\\-]*)@@, killing (?P<affected>[\\d,]*) million infected\\.$").unwrap();
  static ref MOVE:Regex = Regex::new("^@@(?P<nation>[a-z0-9_\\-]*)@@ relocated from %%(?P<from>[a-z0-9_\\-]*)%% \
  to %%(?P<to>[a-z0-9_\\-]*)%%\\.$").unwrap();
}
impl<'a> ZEvent<'a> {
  pub fn from_event(e: &'a Event) -> Option<Self> {
    let id = e.id;
    let timestamp = e.timestamp;
    let from;
    let to;
    let event;
    if let Some(c) = CURE.captures(&e.text) {
      from = c.name("from").unwrap().as_str();
      to = c.name("to").unwrap().as_str();
      let affected = c["affected"]
        .split(',')
        .collect::<String>()
        .parse::<usize>()
        .unwrap();
      let level = match &c["level"] {
        "I" => 1,
        "II" => 2,
        "III" => 3,
        "IV" => 4,
        "V" => 5,
        s => panic!("Invalid numeral {}", s),
      };
      let restored = c.name("restore").is_some();
      event = EventType::Cure {
        affected,
        level,
        restored,
      };
    } else if let Some(c) = ZOMBIE.captures(&e.text) {
      from = c.name("from").unwrap().as_str();
      to = c.name("to").unwrap().as_str();
      let level = c.name("level").unwrap().as_str();
      let affected = c["affected"]
        .split(',')
        .collect::<String>()
        .parse::<usize>()
        .unwrap();
      let converted = c.name("convert").is_some();
      event = EventType::Zombie {
        level,
        converted,
        affected,
      }
    } else if let Some(c) = KILL.captures(&e.text) {
      from = c.name("from").unwrap().as_str();
      to = c.name("to").unwrap().as_str();
      let affected = c["affected"]
        .split(',')
        .collect::<String>()
        .parse::<usize>()
        .unwrap();
      let level = c["level"].parse().unwrap();
      event = EventType::Kill { affected, level };
    } else if let Some(c) = MOVE.captures(&e.text) {
      from = c.name("from").unwrap().as_str();
      to = c.name("to").unwrap().as_str();
      event = EventType::Move {
        nation: c.name("nation").unwrap().as_str(),
      }
    } else {
      return None;
    }
    Some(ZEvent {
      id,
      timestamp,
      from,
      to,
      event,
    })
  }
  pub fn to_graph(events: &[Self]) -> EventGraph<'_> {
    let mut graph = Graph::with_capacity(1500, events.len());
    let mut index_map = BTreeMap::new();
    let mut move_map = BTreeMap::new();
    for event in events {
      if event.event.is_attack() {
        let start = if let Some(idx) = index_map.get(event.from) {
          *idx
        } else {
          let idx = graph.add_node(event.from);
          index_map.insert(event.from, idx);
          idx
        };
        let end = if let Some(idx) = index_map.get(event.to) {
          *idx
        } else {
          let idx = graph.add_node(event.to);
          index_map.insert(event.to, idx);
          idx
        };
        graph.add_edge(start, end, event);
      } else if let EventType::Move { nation } = event.event {
        move_map
          .get_mut(nation)
          .map(|v: &mut Vec<_>| v.push(event))
          .or_else(|| {
            drop(move_map.insert(nation, vec![event]));
            Some(())
          });
      } else {
        unreachable!()
      }
    }
    EventGraph {
      graph,
      index_map,
      move_map,
    }
  }
}
use petgraph::graph::NodeIndex;
use petgraph::Graph;
use std::collections::BTreeMap;
pub struct EventGraph<'a> {
  pub index_map: BTreeMap<&'a str, NodeIndex>,
  pub move_map: BTreeMap<&'a str, Vec<&'a ZEvent<'a>>>,
  pub graph: Graph<&'a str, &'a ZEvent<'a>>,
}
use petgraph::Direction;
impl<'a> EventGraph<'a> {
  pub fn get_stats(&self) -> BTreeMap<&'a str, EventStats> {
    let mut stat_map = BTreeMap::new();
    for (&nation, &i) in &self.index_map {
      let incoming = self.graph.edges_directed(i, Direction::Incoming);
      let outgoing = self.graph.edges_directed(i, Direction::Outgoing);
      let incoming_sum: EventStats = incoming
        .map(|e| *e.weight())
        .map(|e| e.event.stats_incoming())
        .sum();
      let outgoing_sum = outgoing
        .map(|e| *e.weight())
        .map(|e| e.event.stats_outgoing())
        .sum();
      stat_map.insert(nation, incoming_sum + outgoing_sum);
    }
    stat_map
  }
  pub fn get_stats_regex(&self, regex: &Regex) -> EventStats {
    self
      .index_map
      .iter()
      .filter(|(&nation, _)| regex.is_match(nation))
      .map(|(_, &i)| {
        let outgoing: EventStats = self
          .graph
          .edges_directed(i, Direction::Outgoing)
          .map(|e| e.weight().event.stats_outgoing())
          .sum();
        let incoming: EventStats = self
          .graph
          .edges_directed(i, Direction::Incoming)
          .map(|e| e.weight().event.stats_incoming())
          .sum();
        outgoing + incoming
      })
      .sum()
  }
}
use derive_more::*;
use serde::Deserialize;
use serde::Serialize;
#[derive(
  Eq,
  PartialEq,
  Ord,
  PartialOrd,
  Debug,
  Clone,
  Default,
  Add,
  Sum,
  Sub,
  Mul,
  Div,
  Serialize,
  Deserialize,
)]
pub struct EventStats {
  pub missiles_used: usize,
  pub others_cured: usize,
  pub hordes_used: usize,
  pub others_zombified: usize,
  pub tzes_used: usize,
  pub others_killed: usize,
  pub hit_by_missiles: usize,
  pub cured_by_others: usize,
  pub hit_by_hordes: usize,
  pub zombified_by_others: usize,
  pub hit_by_tzes: usize,
  pub killed_by_others: usize,
  pub min_time: usize,
}
#[derive(Serialize, Deserialize)]
pub struct NationData<'a> {
  pub nation: &'a str,
  #[serde(flatten)]
  pub data: EventStats,
}

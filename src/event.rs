use crate::happenings::Event;
use educe::Educe;
use regex::Regex;
#[derive(Eq, PartialEq, Clone, PartialOrd, Ord, Debug, Copy)]
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
    !matches!(self, EventType::Move { .. })
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
  fn to_graph(events: &[Self]) -> Graph<&'a str, Self> {
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
        graph.add_edge(start, end, *event);
      } else {
        if let EventType::Move { nation } = event.event {
          move_map
            .get_mut(nation)
            .map(|v: &mut Vec<_>| v.push(event))
            .or_else(|| Some(drop(move_map.insert(nation, vec![event]))));
        } else {
          unreachable!()
        }
      }
    }
    graph
  }
}
use petgraph::Graph;
use std::collections::BTreeMap;

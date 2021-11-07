use crate::happenings::Event;
use educe::Educe;
use regex::Regex;
#[derive(Eq, PartialEq, Clone, PartialOrd, Ord, Debug)]
pub enum EventType {
  Zombie {
    level: String,
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
    nation: String,
  },
}
#[derive(Educe, Eq, Debug)]
#[educe(PartialEq, PartialOrd, Ord)]
pub struct ZEvent {
  #[educe(PartialEq(ignore), PartialOrd(ignore), Ord(ignore))]
  pub id: u64,
  pub timestamp: u64,
  pub from: String,
  pub to: String,
  pub event: EventType,
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
impl ZEvent {
  pub fn from_event(e: &Event) -> Option<Self> {
    let id = e.id;
    let timestamp = e.timestamp;
    let from;
    let to;
    let event;
    if let Some(c) = CURE.captures(&e.text) {
      from = c["from"].to_owned();
      to = c["to"].to_owned();
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
      from = c["from"].to_owned();
      to = c["to"].to_owned();
      let level = c["level"].to_owned();
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
      from = c["from"].to_owned();
      to = c["to"].to_owned();
      let affected = c["affected"]
        .split(',')
        .collect::<String>()
        .parse::<usize>()
        .unwrap();
      let level = c["level"].parse().unwrap();
      event = EventType::Kill { affected, level };
    } else if let Some(c) = MOVE.captures(&e.text) {
      from = c["from"].to_owned();
      to = c["to"].to_owned();
      event = EventType::Move {
        nation: c["nation"].to_owned(),
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
}

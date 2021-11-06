mod happenings;
use educe::Educe;
use happenings::Event;
use quick_xml::de::from_str;
use quick_xml::se::to_string;
use regex::Regex;
use std::error::Error;
use std::fs::read_to_string;
use std::fs::write;
use std::io::ErrorKind;
fn main() -> Result<(), Box<dyn Error>> {
  let file = read_to_string("../happenings.xml");
  let activities;
  if matches!(file, Err(ref e) if e.kind() == ErrorKind::NotFound) {
    activities = from_str::<Vec<Event>>(&read_to_string("../activities.xml")?)?;
  } else {
    let file = from_str::<Vec<Event>>(&file?)?;
    let regex = Regex::new("ravage||cleanse||struck||relocated")?;
    activities = file
      .into_iter()
      .filter(|e| regex.is_match(&e.text))
      .collect();
    write("../activites.xml", &to_string(&activities)?)?
  }
  Ok(())
}
#[derive(Eq, PartialEq, Clone, PartialOrd, Ord, Debug)]
enum EventType {
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
struct ZEvent {
  #[educe(PartialEq(ignore), PartialOrd(ignore), Ord(ignore))]
  id: u64,
  timestamp: u64,
  from: String,
  to: String,
  event: EventType,
}
lazy_static::lazy_static! {
  static ref CURE:Regex = Regex::new(
    "@@(?P<to>[a-z0-9_])@@ was struck by a Mk (?P<level>[IV]{1,3}) \\([a-zA-Z]*\\) \
    Cure Missile from @@(?P<from>)@@, curing (?P<affected>[\\d,]*) million infected\\.| (p<restore>\
    and restoring to a zombie researcher!)"
  ).unwrap();
  static ref ZOMBIE:Regex = Regex::new("@@(?P<to>[a-z0-9_])@@ was ravaged by a Zombie \
    (?P<level>[a-zA-Z]*) Horde from @@(?P<from>)@@, infecting (?P<affected>[\\d,]*) \
    million survivors\\.|(?P<convert> and converting to a zombie exporter! Oh no!)"
  ).unwrap();
}
impl ZEvent {
  fn from_event(e: Event) -> Option<Self> {
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

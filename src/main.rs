mod happenings;
use event::NationData;
use event::ZEvent;
use happenings::Event;
use quick_xml::de::from_str;
use quick_xml::se::to_string;
use regex::Regex;
use std::error::Error;
use std::fs::read_to_string;
use std::fs::write;
use std::io::ErrorKind;
mod event;
fn main() -> Result<(), Box<dyn Error>> {
    let file = read_to_string("happenings.xml");
    let activities;
    if matches!(file, Err(ref e) if e.kind() == ErrorKind::NotFound) {
        activities = from_str::<Vec<Event>>(&read_to_string("activities.xml")?)?;
    } else {
        let file = from_str::<Vec<Event>>(&file?)?;
        let regex = Regex::new("(ravage)|(cleanse)|(struck)|(relocated)")?;
        activities = file
            .into_iter()
            .filter(|e| regex.is_match(&e.text))
            .collect();
        write("activities.xml", &to_string(&activities)?)?;
    }
    let mut events = activities
        .iter()
        .filter_map(ZEvent::from_event)
        .collect::<Vec<_>>();
    events.sort_unstable();
    events.dedup();
    let mut write = csv::Writer::from_path("zdata.csv")?;
    let graph = ZEvent::to_graph(&events);
    let mut map = graph.get_stats();
    let forest = map.values().cloned().sum();
    map.insert("forest", forest);
    let can = graph.get_stats_regex(&Regex::new(r"can\-([0-9]+)|(founder)")?);
    map.insert("can-*", can);
    let rock = graph.get_stats_regex(&Regex::new(r"rock_([a-z_]+)")?);
    map.insert("rock-*", rock);
    let haven = graph.get_stats_regex(&Regex::new(r"[a-z]+_haven")?);
    map.insert("haven-*", haven);
    map.into_iter()
        .map(|(nation, data)| NationData { nation, data })
        .try_for_each(|n| write.serialize(n))?;
    Ok(())
}

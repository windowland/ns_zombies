mod happenings;
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
    let regex = Regex::new("ravage||cleanse||struck||relocated")?;
    activities = file
      .into_iter()
      .filter(|e| regex.is_match(&e.text))
      .collect();
    write("activites.xml", &to_string(&activities)?)?
  }
  Ok(())
}

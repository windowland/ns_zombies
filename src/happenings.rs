use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub struct Event {
  #[serde(rename = "id")]
  pub id: u64,
  pub timestamp: u64,
  pub text: String,
}

use std::sync::Arc;

use serde::Serialize;

use super::*;

#[derive(Debug, Serialize)]
struct Pinged {
    count: u32,
}

impl StreamEvent for Pinged {
    const FRAME: &'static str = "pinged";
}

#[derive(Debug, Serialize)]
struct Other;

impl StreamEvent for Other {
    const FRAME: &'static str = "other";
}

#[test]
fn a_stream_event_announces_its_frame_name_and_its_fields_as_the_payload() {
    let event: PublishedEvent = Arc::new(Pinged { count: 3 });
    assert_eq!(event.name(), "pinged");
    assert_eq!(event.payload(), serde_json::json!({ "count": 3 }));
}

#[test]
fn a_subscriber_reads_only_the_kind_of_event_it_asks_for() {
    let event: PublishedEvent = Arc::new(Pinged { count: 3 });
    assert_eq!(event.downcast::<Pinged>().map(|pinged| pinged.count), Some(3));
    assert!(event.downcast::<Other>().is_none());
}

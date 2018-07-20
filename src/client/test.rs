use super::*;

use mockito::SERVER_URL;
use mocktopus::mocking::{MockResult, Mockable};

// in unit tests call to mockito SERVER_URL (localhost) in case mocktopus is not used by an unit_test
impl Client {
    pub fn new(http_client: reqwest::Client) -> Self {
        let seattle_client = seattle_data::Client::new(
            http_client.clone(),
            String::from(SERVER_URL),
            String::from("SEATTLE_TOKEN"),
        );
        let bus_client = bus::Client::new(
            http_client,
            String::from(SERVER_URL),
            String::from("BUS_KEY"),
        );

        Client {
            seattle_client,
            bus_client,
        }
    }
}

#[test]
fn info() {
    let subject = Client::new(reqwest::Client::new());
    let expected = String::from("{}");

    let mut query = None;
    unsafe {
        seattle_data::Client::crime.mock_raw(|_, q| {
            query = Some(q.clone());
            MockResult::Return(Ok(expected.clone()))
        })
    }

    let location = Location {
        latitude: 32.2,
        longitude: 67.23,
    };
    let actual = subject
        .info(location)
        .expect("expected a succesful crime response");

    assert_eq!(actual, expected);
    let actual_query = query.expect("seattle_data::Client::request not called");
    let expected_query = seattle_data::Query::new(location).and(Local::now() - Duration::days(180));
    assert_eq!(actual_query, expected_query);
}

#[test]
fn bus_stops() {
    let subject = Client::new(reqwest::Client::new());
    let mut query = None;
    let expected_stops = vec![bus::StopInfo {
        id: String::from("1_1234"),
        direction: String::from("N"),
        name: String::from("some bus"),
        lat: 34.3199,
        lon: 23.12005,
    }];
    unsafe {
        bus::Client::stops.mock_raw(|_, q| {
            query = Some(q.clone());
            MockResult::Return(Ok(expected_stops.clone()))
        });
    }

    let area = Area {
        lat: 34.32,
        lon: 23.12,
        lat_span: 0.002,
        lon_span: 0.0005,
    };
    let actual_stops = subject
        .bus_stops(area)
        .expect("expected a succesful bus stop response");

    assert_eq!(actual_stops, expected_stops);
    let actual_query = query.expect("'bus::Client::stops' not called");
    let expected_query = bus::StopsQuery {
        lat: 34.32,
        lon: 23.12,
        lat_span: 0.002,
        lon_span: 0.0005,
        max_count: 20,
    };
    assert_eq!(actual_query, expected_query)
}

#[test]
fn bus_stop_status() {
    let subject = Client::new(reqwest::Client::new());
    let mut bus_stop_id = None;
    let departure_info = bus::Departures {
        buses: vec![bus::Status {
            headsign: String::from("MAGNOLIA PKWY"),
            route: String::from("26E"),
            predicted_time: 222324334,
            scheduled_time: 232343432,
        }],
        stop: bus::StopInfo {
            direction: String::from("N"),
            id: String::from("1_2345"),
            name: String::from("some name"),
            lat: 233.232233,
            lon: -123.23322,
        },
    };
    unsafe {
        bus::Client::departures.mock_raw(|_, b| {
            bus_stop_id = Some(b.clone());
            MockResult::Return(Ok(departure_info.clone()))
        });
    }
    let actual_status = subject
        .bus_stop_status("3_232")
        .expect("expected a succesful bus stop status");
    let expected_status = BusStopStatus {
        buses: departure_info.buses,
        info: departure_info.stop,
    };
    assert_eq!(actual_status, expected_status);
    let bus_stop_id = bus_stop_id.expect("'bus::Client::departures' not called");
    assert_eq!(bus_stop_id, "3_232");
}

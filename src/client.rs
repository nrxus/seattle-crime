mod bus;
mod seattle_crime;

pub use self::{
    bus::{Status as BusStatus, StopInfo as BusStopInfo},
    seattle_crime::Crime,
};
use crate::api::Area;

#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Debug)]
pub struct BusStopStatus {
    #[serde(flatten)]
    pub info: bus::StopInfo,
    pub buses: Vec<bus::Status>,
    pub related_crimes: Vec<Crime>,
    pub unrelated_crimes: Vec<Crime>,
}

#[cfg_attr(test, faux::create)]
pub struct Client {
    crime_service: seattle_crime::Service,
    bus_client: bus::Client,
}

#[cfg_attr(test, faux::methods)]
impl Client {
    pub fn new(crime_service: seattle_crime::Service, bus_client: bus::Client) -> Self {
        Client {
            crime_service,
            bus_client,
        }
    }

    pub fn bus_stops(&self, area: &Area) -> Result<Vec<bus::StopInfo>, String> {
        self.bus_client.stops(&bus::StopsQuery {
            lat: area.lat,
            lon: area.lon,
            lat_span: area.lat_span,
            lon_span: area.lon_span,
            max_count: area.limit.unwrap_or(20),
        })
    }

    pub fn bus_stop_status(&self, stop_id: &str) -> Result<BusStopStatus, String> {
        let departures = self.bus_client.departures(stop_id)?;
        let crime_data = self.crime_service.crime_nearby(seattle_crime::Location {
            lat: departures.stop.lat,
            lon: departures.stop.lon,
        })?;

        Ok(BusStopStatus {
            info: departures.stop,
            buses: departures.buses,
            related_crimes: crime_data.related_crimes,
            unrelated_crimes: crime_data.unrelated_crimes,
        })
    }
}

// not mocked because we explicitely do not want a real http client under tests
impl Client {
    pub fn from_http_client(http_client: reqwest::blocking::Client) -> Self {
        fn expect_env(name: &str) -> String {
            use std::env;
            env::var(name).unwrap_or_else(|_| panic!("'{}' ENV VARIABLE IS REQUIRED", name))
        }

        let crime_service = {
            use self::seattle_crime::{data, geo};
            let data_client = {
                let token = expect_env("SEATTLE_API_KEY");
                let host = "https://data.seattle.gov".to_string();
                data::Client::new(http_client.clone(), host, token)
            };
            let geo_client = {
                let host = "https://gisrevprxy.seattle.gov".to_string();
                geo::Client::new(http_client.clone(), host)
            };
            seattle_crime::Service::new(data_client, geo_client)
        };
        let bus_client = {
            let key = expect_env("ONEBUSAWAY_API_KEY");
            let host = "http://api.pugetsound.onebusaway.org".to_string();
            bus::Client::new(http_client, host, key)
        };

        Client::new(crime_service, bus_client)
    }
}

#[cfg(all(test, not(feature = "integration")))]
mod bus_test;

#[cfg(all(test, not(feature = "integration")))]
mod test;

// user: ROOTNAME
// password: CHANGEME123
// Initial Organization: MUIC
// Initial Bucket: storage

// API token: rg4g3-miZbuq7JcMEOVa0BPPanp6nGvkFi6gGVTSweQRCJTzORPEEiWpTpBso0dZOnoM6zgy2ykFz9SM6HBF4Q==


// docker run \
//  --name influxdb2 \
//  --publish 8086:8086 \
//  --mount type=volume,source=influxdb2-data,target=/var/lib/influxdb2 \
//  --mount type=volume,source=influxdb2-config,target=/etc/influxdb2 \
//  --env DOCKER_INFLUXDB_INIT_MODE=setup \
//  --env DOCKER_INFLUXDB_INIT_USERNAME=ADMIN_USERNAME \
//  --env DOCKER_INFLUXDB_INIT_PASSWORD=ADMIN_PASSWORD \
//  --env DOCKER_INFLUXDB_INIT_ORG=ORG_NAME \
//  --env DOCKER_INFLUXDB_INIT_BUCKET=BUCKET_NAME \
//  influxdb:2



use chrono::{DateTime, Utc};
use influxdb::{Client, Error, InfluxDbWriteable, ReadQuery, Timestamp};
#[derive(Deserialize)]


// IfluxDB keep time series data into buckets. Bucket contains multible measurement<- contain tags and fields
// Measurement <- logical grouping with multiple tages and fields
    // Tags <- key value pairs, storing metadata info (not change often) like location, host station
    // Fields <- key value pairs, storing data (change often) like temperature, humidity
    // Timestamp <- time

// Schema that I have in mind is one bucket
//two measurements
//Incoming IP:
    //Tags: 
        // AS, Country
    //Fields:
        // IP

//Outgoing IP
    //Tags:
        // AS, Country
    //Fields:
        // IP



#[derive(InfluxDbWriteable)]
struct Package{
    time: DateTime<Utc>,
    IP: String,
    #[influxdb(tag)]
    AS: String,
    Country: String,
}

enum IPtype {
    Incoming,
    Outgoing,
}

fn create_client(bucket:&str) -> Client {
    let client = Client::new("http://localhost:8086", bucket);
    client
}

pub fn create_db(db_name: &str) -> Result<(), influxdb::Error> {
    create_db(db_name)
}

async fn write_data(client: Client, pack: Package, iptype: IPtype) -> Result<(), influxdb::Error> {
    let mut write_query;
    match iptype {
        IPtype::Incoming => {
            write_query = Query::write_query(pack.into_query("incoming"));
        }
        IPtype::Outgoing => {
             write_query = Query::write_query(pack.into_query("outgoing"));
        }
    }
    client.query(write_query).await?;
    Ok(())
}



fn main() {
    // let client = Client::new("http://localhost:8086", "test");
    // let query = Query::raw_read_query(
    //     "SELECT temperature FROM /weather_[a-z]*$/ WHERE time > now() - 1m ORDER BY DESC",
    // );
    // let mut db_result = client.json_query(query).await?;
    // let _result = db_result
    //     .deserialize_next::<WeatherWithoutCityName>()?
    //     .series
    //     .into_iter()
    //     .map(|mut city_series| {
    //         let city_name =
    //             city_series.name.split("_").collect::<Vec<&str>>().remove(2);
    //         Weather {
    //             weather: city_series.values.remove(0),
    //             city_name: city_name.to_string(),
    //         }
    //     })
    //     .collect::<Vec<Weather>>();


}






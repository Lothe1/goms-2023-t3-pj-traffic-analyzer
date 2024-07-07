use chrono::{DateTime, Utc};
use influxdb::{ Error, InfluxDbWriteable, ReadQuery, Timestamp};
use influxdb::{Client, Query};
use serde::Deserialize;


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

async fn write_data(client: Client, pack: Package, iptype: IPtype) -> Result<(), influxdb::Error> {
    let mut write_query;
    match iptype {
        IPtype::Incoming => {
            write_query = pack.into_query("incoming");
        }
        IPtype::Outgoing => {
            write_query = pack.into_query("outgoing");
        }
    }
    client.query(write_query).await?;

    // Let's see if the data we wrote is there
    let read_query = ReadQuery::new("SELECT * FROM incoming");

    let read_result = client.query(read_query).await?;
    println!("{}", read_result);

    Ok(())
}



#[tokio::main(flavor = "current_thread")]
// This attribute makes your main function asynchronous
async fn main()  ->  Result<(), Box<dyn std::error::Error>> { // Use Box<dyn Error> for a general error type
    let client = Client::new("http://localhost:8086", "test")
        .with_token("18uSAEcBTO7qpd50qUwntr1NtTru3oZRx7yPfprA57qWhWkE3BlV_fXIB6TTZcLObPRbK0OdrMc27uGVXRWAHg==")
        ;

    let test_pack = Package {
        time: Utc::now(),
        IP: "192.168.1.1/69".to_string(),
        AS: "AS142".to_string(),
        Country: "Thailand".to_string(),
    };

    let test = write_data(client, test_pack, IPtype::Incoming).await?;


    Ok(())

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







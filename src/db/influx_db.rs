use chrono::{DateTime, Utc};
use influxdb::{InfluxDbWriteable};
use influxdb::{Client, Query};
use serde::{Deserialize, Serialize};

use super::ip_lookup::IPtype;

#[derive(Clone, Debug)]
#[derive(InfluxDbWriteable)]
#[derive(Serialize, Deserialize)]
pub struct Package{
    pub(crate) time: DateTime<Utc>,
    pub(crate) IP: String,
    pub(crate) AS: String,
    pub(crate) location: String,
    pub(crate) bytes_count: i32,
}



#[derive(Serialize, Deserialize, Debug)]
pub struct CustomMessage{
    pub(crate) package: Package,
    pub(crate) iptype: IPtype,
}



pub fn create_client(bucket:&str, token: &str) -> Client {
    let client = Client::new("http://localhost:8086", bucket)
        .with_token(token);
    client
}
async fn read_all_table_query(client: Client, table: &str) -> Result<String, influxdb::Error> {
    let query = <dyn Query>::raw_read_query(
        format!("SELECT * FROM {}", table).as_str(),
    );
    let read_result = client.query(query).await?;
    return Ok(read_result.to_string());
}
pub async fn write_data(client: Client, pack: Package, iptype: IPtype) -> Result<(), influxdb::Error> {
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
    Ok(())
}
pub async fn make_package(time: DateTime<Utc>, IP: &str, AS: &str, Country: &str, bytes: i32) -> Package {
    let pack = Package {
        time: time,
        IP: IP.to_string(),
        AS: AS.to_string(),
        location: Country.to_string(),
        bytes_count: bytes,
    };
    pack
}
async fn write_datas(client: Client, packvec: Vec<Package>, iptype: IPtype) -> Result<(), influxdb::Error> {
    let mut vec_query = Vec::new();
    for pack in packvec {
        let mut write_query;
        match iptype {
            IPtype::Incoming => {
                write_query = pack.into_query("incoming");
            }
            IPtype::Outgoing => {
                write_query = pack.into_query("outgoing");
            }
        }
        vec_query.push(write_query);
    }
    client.query(vec_query).await?;
    Ok(())
}
async fn makeThaipackage()-> Result<Vec<Package>, influxdb::Error>{
    let mut packvec = Vec::new();
    packvec.push(
        make_package(
        Utc::now(),
        "Thai_IP1.1",
        "THAI_AS1",
        "TH",
        16
    )
        .await
    );

    packvec.push(
        make_package(
            Utc::now() - chrono::Duration::minutes(2),
            "Thai_IP1.2",
            "THAI_AS1",
            "TH",
            33
        )
            .await
    );

    packvec.push(
        make_package(
            Utc::now() - chrono::Duration::minutes(14),
            "Thai_IP1.3",
            "THAI_AS1",
            "TH",
            11
        )
            .await
    );

    packvec.push(
        make_package(
            Utc::now() - chrono::Duration::days(4),
            "Thai_IP2.1",
            "THAI_AS2",
            "TH",
            88
        )
            .await
    );
    packvec.push(
        make_package(
            Utc::now() - chrono::Duration::days(1),
            "Thai_IP2.2",
            "THAI_AS2",
            "TH",
            99
        )
            .await
    );

    packvec.push(
        make_package(
            Utc::now() - chrono::Duration::days(2),
            "Thai_IP3.1",
            "THAI_AS3",
            "TH",
            451
        )
            .await
    );

    return Ok(packvec);

}
async fn makeAustraliapackage()-> Result<Vec<Package>, influxdb::Error>{
    let mut packvec = Vec::new();
    packvec.push(
        make_package(
            Utc::now(),
            "Aus_IP1.1",
            "Aus_AS1",
            "AT",
            123
        )
            .await
    );

    packvec.push(
        make_package(
            Utc::now(),
            "Aus_IP1.2",
            "Aus_AS1",
            "AT",
            14515
        )
            .await
    );

    packvec.push(
        make_package(
            Utc::now(),
            "Aus_IP1.3",
            "Aus_AS1",
            "AT",
            44
        )
            .await
    );

    packvec.push(
        make_package(
            Utc::now(),
            "Aus_IP2.1",
            "Aus_AS2",
            "AT",
            22
        )
            .await
    );
    packvec.push(
        make_package(
            Utc::now()+chrono::Duration::days(1),
            "Aus_IP2.2",
            "Aus_AS2",
            "AT",
            40
        )
            .await
    );

    packvec.push(
        make_package(
            Utc::now()+ chrono::Duration::days(4),
            "Aus_IP3.1",
            "Aus_AS3",
            "AT",
            210
        )
            .await
    );

    packvec.push(
        make_package(
            Utc::now()+ chrono::Duration::days(2),
            "Aus_IP3.2",
            "Aus_AS4",
            "AT",
            339
        )
            .await
    );
    return Ok(packvec);
}



// I tested with this main function
// #[tokio::main(flavor = "current_thread")]
// // This attribute makes your main function asynchronous

// async fn main()  ->  Result<(), Box<dyn std::error::Error>> { // Use Box<dyn Error> for a general error type
//     let token = "duuW20S6Ki63k5EIEx6sahpMQ5QKDbo5eYNbN1Ux91Hx00oY9xAQDwIaU-JnoNg7wRYkww457heGfPgcX7Y-UA==";

//     let client = Client::new("http://localhost:8086", "db")
//         .with_token(token);

//     let packvecThai = makeThaipackage().await?;
//     let packvecAus = makeAustraliapackage().await?;
//     write_datas(client.clone(), packvecThai.clone(), IPtype::Outgoing).await?;
//     write_datas(client.clone(), packvecAus.clone(), IPtype::Outgoing).await?;
//     write_datas(client.clone(), packvecThai, IPtype::Incoming).await?;
//     write_datas(client.clone(), packvecAus, IPtype::Incoming).await?;

//     //
//     // let res = read_all_table_query(client.clone(), "incoming").await?;
//     // let res2 = read_all_table_query(client.clone(), "outgoing").await?;
//     // println!("{}", res);
//     // println!("{}", res2);
    
//     let res = read_all_table_query(client.clone(), "incoming").await?;
//     println!("{}", res);

//     Ok(())

   
// }


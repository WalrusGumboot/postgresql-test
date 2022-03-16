use tokio;
use tokio_postgres::{NoTls, Error};

use futures::future;
use std::future::Future;

use openssl::ssl::{SslConnector, SslMethod};
use postgres_openssl::MakeTlsConnector;

struct Product {
    ean13: i64,
    naam: String,
    beschrijving: String,
    prijs: i16,
    gewicht: i16,
    merk: String
}

impl Product {
    fn naar_query (&self) -> String {
        format!("'{}'::bigint, '{}'::text, '{}'::text, '{}'::smallint, '{}'::smallint, '{}'::text",
                self.ean13,
                self.naam,
                self.beschrijving,
                self.prijs,
                self.gewicht,
                self.merk)

    }
}

async fn nieuw_product(p: Product, client: &tokio_postgres::Client) -> Result<(), Error> {
    //INSERT INTO public.producten (
    //ean13, naam, beschrijving, prijs, gewicht, merk) VALUES (
    //'8710835839813'::bigint, '96 Moeilijke Sudokupuzzels'::text, 'Een bundel sudokus.'::text, '149'::smallint, '230'::smallint, 'Denksport'::text)
    //returning ean13;

    let res = client.query(
        format!("INSERT INTO public.producten (ean13, naam, beschrijving, prijs, gewicht, merk) VALUES ({}) returning ean13;", p.naar_query()).as_str(),
        &[]).await;

    match res {
        Ok(_) => Ok(()),
        Err(e) => Err(e)
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Connect to the database.
    let mut ssl_builder = SslConnector::builder(SslMethod::tls()).unwrap();
    ssl_builder.set_ca_file("db_certificate.cer").unwrap();
    let connector = MakeTlsConnector::new(ssl_builder.build());

    let (client, connection) =
        //tokio_postgres::connect("host=localhost user=simeon dbname=AlbertHeijn", connector).await?;
        tokio_postgres::connect("host=db-postgresql-ams3-52327-do-user-9835208-0.b.db.ondigitalocean.com user=doadmin dbname=defaultdb sslmode=require password=JzyQ6jOrMYxIPz1s port=25060",
                                connector).await.unwrap();

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let rows = client
        .query("SELECT * FROM public.producten ORDER BY id ASC", &[])
        .await.unwrap();

    let product1 = Product {
        ean13: 8712417730174,
        naam: String::from("Blauwe A4 ringmap"),
        beschrijving: String::from("Dubbelringse kartonnen ringmap op A4-formaat, gemaakt van gerecycleerd karton."),
        prijs: 129,
        gewicht: 50,
        merk: String::from("DLP Office")
    };

    let product2 = Product {
        ean13: 9781847497192,
        naam: String::from("The Trial - Franz Kafka"),
        beschrijving: String::from("Een klassieker van de Duitse literatuur in paperback-uitgave."),
        prijs: 799,
        gewicht: 90,
        merk: String::from("Alma Classics")
    };

    println!("Verbinding met database ingesteld; {} items gevonden.", rows.len());
    let ean13: String = rows[0].get("test_veld");
    println!("EANs: {:?}", ean13);

    let producten: Vec<Product> = vec![product1, product2];

    future::try_join_all(producten.into_iter().map(|p| {
        nieuw_product(p, &client)
    })).await;

    Ok(())
}

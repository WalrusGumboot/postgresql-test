use tokio;
use tokio_postgres::{NoTls, Error};

use futures::future;
use std::future::Future;



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
    let (client, connection) =
        tokio_postgres::connect("host=localhost user=simeon dbname=AlbertHeijn", NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let rows = client
        .query("SELECT * FROM public.producten ORDER BY ean13 ASC", &[])
        .await?;

    // let p = Product {
    //     ean13: 8712417730174,
    //     naam: String::from("Blauwe A4 ringmap"),
    //     beschrijving: String::from("Dubbelringse kartonnen ringmap op A4-formaat, gemaakt van gerecycleerd karton."),
    //     prijs: 129,
    //     gewicht: 50,
    //     merk: String::from("DLP Office")
    // };

    // nieuw_product(p, &client).await.unwrap();

    println!("Verbinding met database ingesteld; {} items gevonden.", rows.len());
    let ean13: i64 = rows[0].get("ean13");
    println!("EANs: {:?}", ean13);

    let producten: Vec<Product> = Vec::new();

    future::try_join_all(producten.into_iter().map(|p| {
        nieuw_product(p, &client)
    })).await;

    Ok(())
}

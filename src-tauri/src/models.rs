use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize)]
pub struct Artwork {
    pub id: String, pub code: String, pub title: String,
    pub year: Option<String>, pub technique: Option<String>, pub support: Option<String>,
    pub width_cm: Option<f64>, pub height_cm: Option<f64>, pub depth_cm: Option<f64>,
    pub description: Option<String>, pub location: Option<String>, pub status: String,
    pub image_path: Option<String>, pub created_at: String, pub updated_at: String,
}
#[derive(Debug, Deserialize)]
pub struct ArtworkInput {
    pub id: Option<String>, pub code: String, pub title: String,
    pub year: Option<String>, pub technique: Option<String>, pub support: Option<String>,
    pub width_cm: Option<f64>, pub height_cm: Option<f64>, pub depth_cm: Option<f64>,
    pub description: Option<String>, pub location: Option<String>, pub status: String,
}
#[derive(Clone, Debug, Serialize)]
pub struct Collector {
    pub id: String, pub name: String, pub email: Option<String>, pub phone: Option<String>,
    pub address: Option<String>, pub notes: Option<String>, pub created_at: String, pub updated_at: String,
}
#[derive(Debug, Deserialize)]
pub struct CollectorInput {
    pub id: Option<String>, pub name: String, pub email: Option<String>,
    pub phone: Option<String>, pub address: Option<String>, pub notes: Option<String>,
}
#[derive(Debug, Serialize)]
pub struct Sale {
    pub id: String, pub artwork_id: String, pub collector_id: String, pub amount_cents: i64,
    pub currency: String, pub sold_at: String, pub notes: Option<String>,
    pub artwork_title: String, pub collector_name: String, pub created_at: String,
}
#[derive(Debug, Deserialize)]
pub struct SaleInput {
    pub artwork_id: String, pub collector_id: String, pub amount_cents: i64,
    pub sold_at: String, pub notes: Option<String>,
    #[serde(default)] pub tax_cents: i64,
    #[serde(default)] pub shipping_cents: i64,
}
#[derive(Clone, Debug, Serialize)]
pub struct Certificate {
    pub id: String, pub artwork_id: String, pub serial: String, pub issued_at: String,
    pub artist_name: String, pub collector_name: Option<String>, pub notes: Option<String>,
    pub snapshot_json: String, pub artwork_title: String, pub created_at: String,
}
#[derive(Debug, Deserialize)]
pub struct CertificateInput {
    pub artwork_id: String, pub artist_name: String,
    pub collector_name: Option<String>, pub notes: Option<String>,
}
#[derive(Debug, Serialize)]
pub struct Overview {
    pub artworks: i64, pub collectors: i64, pub sales: i64,
    pub certificates: i64, pub revenue_cents: i64,
}

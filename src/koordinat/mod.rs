//! Koordinat sistemleri — `echarts/src/coord` dizininin karşılığı.

pub mod eksen;
pub mod kartezyen;
pub mod matris;
pub mod paralel;
pub mod takvim;
pub mod tek_eksen;

pub use eksen::ÇalışmaEkseni;
pub use kartezyen::{Dikdörtgen, Kartezyen2B};
pub use matris::{MatrisHücreTürü, MatrisHücreYerleşimi, MatrisYerleşimi};
pub use paralel::{
    ParalelEtkinlik, ParalelGenişletmeDavranışı, ParalelGenişletmeSonucu, ParalelYerleşimi,
    ParalelÇalışmaEkseni, seri_bağlı_mı as paralel_seri_bağlı_mı,
};
pub use takvim::TakvimYerleşimi;
pub use tek_eksen::TekEksenYerleşimi;

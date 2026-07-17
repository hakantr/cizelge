use crate::Nokta;

/// Ekran uzayındaki en yakın noktayı bulur. Eşik dışındaki sonuçlar seçilmez;
/// böylece yoğun seride imleç boşluğa geldiğinde rastgele veri açıklanmaz.
pub fn en_yakin_nokta(noktalar: &[Nokta], hedef: Nokta, azami_uzaklik: f64) -> Option<usize> {
    if !hedef.x.is_finite()
        || !hedef.y.is_finite()
        || !azami_uzaklik.is_finite()
        || azami_uzaklik < 0.0
    {
        return None;
    }
    let esik_kare = azami_uzaklik * azami_uzaklik;
    noktalar
        .iter()
        .enumerate()
        .filter_map(|(sira, nokta)| {
            let dx = nokta.x - hedef.x;
            let dy = nokta.y - hedef.y;
            let uzaklik = dx.mul_add(dx, dy * dy);
            uzaklik.is_finite().then_some((sira, uzaklik))
        })
        .filter(|(_, uzaklik)| *uzaklik <= esik_kare)
        .min_by(|sol, sag| sol.1.total_cmp(&sag.1))
        .map(|(sira, _)| sira)
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn esik_disindaki_nokta_secilmez() {
        let noktalar = [Nokta::yeni(0.0, 0.0), Nokta::yeni(10.0, 10.0)];
        assert_eq!(
            en_yakin_nokta(&noktalar, Nokta::yeni(9.0, 9.0), 2.0),
            Some(1)
        );
        assert_eq!(en_yakin_nokta(&noktalar, Nokta::yeni(5.0, 5.0), 2.0), None);
    }
}

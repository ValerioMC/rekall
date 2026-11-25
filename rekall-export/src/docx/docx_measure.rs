const TWIPS_PER_MM: f64 = 56.692_913;
const EMU_PER_MM: f64 = 36_000.0;

pub(super) struct DocxMeasure;

impl DocxMeasure {
    pub(super) fn twips(millimetres: f64) -> i32 {
        (millimetres * TWIPS_PER_MM).round() as i32
    }

    pub(super) fn emu(millimetres: f64) -> u32 {
        (millimetres * EMU_PER_MM).round() as u32
    }

    pub(super) fn half_points(points: f64) -> usize {
        (points * 2.0).round() as usize
    }
}


pub fn quantize_to_kg(amount: i64) -> i64 {
    const KG: i64 = 10_000;
    let kg_amount = amount / KG;
    kg_amount * KG
}

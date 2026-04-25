/// Sensible default values for scenario construction.

/// Look up state income tax rate by US state abbreviation.
pub fn state_tax_rate(state: &str) -> f64 {
    match state.to_uppercase().as_str() {
        // No income tax states
        "AK" | "FL" | "NV" | "NH" | "SD" | "TN" | "TX" | "WA" | "WY" => 0.0,
        // States with rates
        "AL" => 0.0500,
        "AZ" => 0.0250,
        "AR" => 0.0390,
        "CA" => 0.0930,
        "CO" => 0.0440,
        "CT" => 0.0699,
        "DE" => 0.0660,
        "GA" => 0.0549,
        "HI" => 0.1100,
        "ID" => 0.0580,
        "IL" => 0.0495,
        "IN" => 0.0305,
        "IA" => 0.0570,
        "KS" => 0.0570,
        "KY" => 0.0400,
        "LA" => 0.0425,
        "ME" => 0.0715,
        "MD" => 0.0575,
        "MA" => 0.0500,
        "MI" => 0.0425,
        "MN" => 0.0985,
        "MS" => 0.0500,
        "MO" => 0.0480,
        "MT" => 0.0575,
        "NE" => 0.0564,
        "NJ" => 0.0897,
        "NM" => 0.0590,
        "NY" => 0.0685,
        "NC" => 0.0450,
        "ND" => 0.0195,
        "OH" => 0.0399,
        "OK" => 0.0475,
        "OR" => 0.0990,
        "PA" => 0.0307,
        "RI" => 0.0599,
        "SC" => 0.0640,
        "UT" => 0.0465,
        "VT" => 0.0875,
        "VA" => 0.0575,
        "WV" => 0.0512,
        "WI" => 0.0753,
        "DC" => 0.0895,
        _ => 0.05, // default fallback
    }
}

/// Rough Social Security benefit estimator.
/// Returns estimated monthly benefit in today's dollars.
pub fn estimate_social_security(annual_income: f64) -> f64 {
    if annual_income <= 70_000.0 {
        annual_income * 0.40 / 12.0
    } else {
        2_500.0 + (annual_income - 70_000.0) * 0.15 / 12.0
    }
}

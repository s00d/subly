//! Decimal money helpers — store and compute amounts without IEEE-754 drift.

use rust_decimal::Decimal;
use serde::Deserializer;
use std::collections::HashMap;
use std::str::FromStr;

pub use rust_decimal::Decimal as Money;

pub const MIGRATION_KV_KEY: &str = "migration:money_decimal_v1";

pub fn from_f64_legacy(v: f64) -> Decimal {
    if !v.is_finite() {
        return Decimal::ZERO;
    }
    Decimal::from_f64_retain(v).unwrap_or_else(|| {
        Decimal::from_str(&format!("{v:.10}")).unwrap_or(Decimal::ZERO)
    })
}

pub fn parse_money_str(s: &str) -> Result<Decimal, String> {
    let t = s.trim();
    if t.is_empty() {
        return Ok(Decimal::ZERO);
    }
    Decimal::from_str(t).map_err(|e| e.to_string())
}

pub fn is_valid_non_negative(d: &Decimal) -> bool {
    d >= &Decimal::ZERO
}

#[allow(dead_code)]
pub fn sum_money<I>(iter: I) -> Decimal
where
    I: IntoIterator<Item = Decimal>,
{
    iter.into_iter().fold(Decimal::ZERO, |a, b| a + b)
}

pub fn rate_map_from_currencies(
    currencies: &[crate::models::CurrencyDoc],
) -> HashMap<String, Decimal> {
    currencies
        .iter()
        .map(|c| {
            let rate = if c.rate <= Decimal::ZERO {
                Decimal::ONE
            } else {
                c.rate
            };
            (c.id.clone(), rate)
        })
        .collect()
}

pub fn convert_to_main(
    amount: Decimal,
    currency_id: &str,
    rates: &HashMap<String, Decimal>,
) -> Decimal {
    let rate = rates.get(currency_id).copied().unwrap_or(Decimal::ONE);
    if rate <= Decimal::ZERO {
        amount
    } else {
        amount / rate
    }
}

pub fn price_per_month(cycle: u8, frequency: u32, price: Decimal) -> Decimal {
    let freq = Decimal::from(if frequency == 0 { 1 } else { frequency });
    match cycle {
        1 => price * Decimal::from_f64_retain(365.25).unwrap_or(Decimal::ONE) / freq
            / Decimal::from(12),
        2 => price * Decimal::from(52) / freq / Decimal::from(12),
        3 => price / freq,
        4 => price / (Decimal::from(12) * freq),
        _ => price,
    }
}

pub fn to_ipc_string(d: &Decimal) -> String {
    d.normalize().to_string()
}

/// Compare FX rates from HTTP APIs (still `f64` at the boundary).
pub fn rates_differ_stored(stored: Decimal, api_rate: f64) -> bool {
    let incoming = from_f64_legacy(api_rate);
    (stored - incoming).abs() > Decimal::new(1, 8)
}

pub fn decimal_from_api_rate(v: f64) -> Decimal {
    from_f64_legacy(v)
}

pub(crate) fn deserialize_flexible<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: Deserializer<'de>,
{
    struct MoneyVisitor;

    impl<'de> serde::de::Visitor<'de> for MoneyVisitor {
        type Value = Decimal;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a money amount as string or number")
        }

        fn visit_str<E>(self, v: &str) -> Result<Decimal, E>
        where
            E: serde::de::Error,
        {
            Decimal::from_str(v.trim()).map_err(E::custom)
        }

        fn visit_f64<E>(self, v: f64) -> Result<Decimal, E>
        where
            E: serde::de::Error,
        {
            Ok(from_f64_legacy(v))
        }

        fn visit_i64<E>(self, v: i64) -> Result<Decimal, E>
        where
            E: serde::de::Error,
        {
            Ok(Decimal::from(v))
        }

        fn visit_u64<E>(self, v: u64) -> Result<Decimal, E>
        where
            E: serde::de::Error,
        {
            Ok(Decimal::from(v))
        }
    }

    deserializer.deserialize_any(MoneyVisitor)
}

pub mod serde_ipc {
    use super::{deserialize_flexible, Money};
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(value: &Money, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        rust_decimal::serde::str::serialize(value, serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Money, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize_flexible(deserializer)
    }
}

pub mod serde_ipc_option {
    use super::{deserialize_flexible, Money};
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(value: &Option<Money>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(v) => rust_decimal::serde::str::serialize(v, serializer),
            None => serializer.serialize_none(),
        }
    }

    #[allow(dead_code)]
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Money>, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct OptVisitor;

        impl<'de> serde::de::Visitor<'de> for OptVisitor {
            type Value = Option<Money>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("optional money amount")
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(None)
            }

            fn visit_some<D2>(self, deserializer: D2) -> Result<Self::Value, D2::Error>
            where
                D2: Deserializer<'de>,
            {
                deserialize_flexible(deserializer).map(Some)
            }
        }

        deserializer.deserialize_option(OptVisitor)
    }
}

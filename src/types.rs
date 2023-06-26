use std::collections::HashMap;

#[derive(Clone, Copy, Debug)]
pub enum SIBaseUnits {
    Second,
    Meter,
    Kilogram,
    Ampere,
    Kelvin,
    Mole,
    Candela,
}

pub const NUM_BASE_UNITS: usize = std::mem::variant_count::<SIBaseUnits>();

impl From<usize> for SIBaseUnits {
    fn from(val: usize) -> Self {
        let variants: [SIBaseUnits; NUM_BASE_UNITS] = [
            SIBaseUnits::Second, SIBaseUnits::Meter,
            SIBaseUnits::Kilogram, SIBaseUnits::Ampere,
            SIBaseUnits::Kelvin, SIBaseUnits::Mole, SIBaseUnits::Candela];
        if val < variants.len() {
            variants[val]
        } else {
            panic!("Invalid value for SIBaseUnits")
        }
    }
}

impl SIBaseUnits {
    pub fn into_usize(&self) -> usize {
        match self {
            SIBaseUnits::Second => 0,
            SIBaseUnits::Meter => 1,
            SIBaseUnits::Kilogram => 2,
            SIBaseUnits::Ampere => 3,
            SIBaseUnits::Kelvin => 4,
            SIBaseUnits::Mole => 5,
            SIBaseUnits::Candela => 6,
        }
    }
}

impl std::fmt::Display for SIBaseUnits {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Type {
    pub scalar_prefix: f64,
    pub si_units: [i32; NUM_BASE_UNITS]
}

pub fn parse_type_comment<'a>(text: &'a str) -> Option<(&'a str, Type)> {
    let v: Vec<&str> = text.split(":").collect();
    if v.len() != 2 {
        return None
    }

    parse_human_type(v[1].trim()).and_then(|t| Some((v[0], t)))
}

pub fn parse_human_type(text: &str) -> Option<Type> {
    let m: HashMap<&str, Type> = HashMap::from([
        ("m", Type{scalar_prefix: 1.0, si_units: [0, 1, 0, 0, 0, 0, 0]}),
        ("cm", Type{scalar_prefix: -2.0, si_units: [0, 1, 0, 0, 0, 0, 0]})
    ]);

    m.get(text).and_then(|x| Some(*x))
}

impl Type {
    // pub fn new() -> Type {
    //     Type{scalar_prefix: 0.0, si_units: [0; NUM_BASE_UNITS]}
    // }
}
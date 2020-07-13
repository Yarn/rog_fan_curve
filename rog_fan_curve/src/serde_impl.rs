
use std::fmt;
use serde::de::{ self, Visitor, Deserialize };
use serde::ser::{ Serialize, Serializer };

use crate::Curve;

struct CurveVisitor;

impl<'de> Visitor<'de> for CurveVisitor {
    type Value = Curve;
    
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a rog_fan_curve config string")
    }
    
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where E: de::Error
    {
        Curve::from_config_str(v)
            .map_err(|err| {
                E::custom(err)
            })
    }
}

impl<'de> Deserialize<'de> for Curve {
    fn deserialize<D>(deserializer: D) -> Result<Curve, D::Error>
        where D: de::Deserializer<'de>
    {
        deserializer.deserialize_str(CurveVisitor)
    }
}

impl Serialize for Curve {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let config = self.as_config_string();
        serializer.serialize_str(&config)
    }
}

#[cfg(test)]
mod tests {
    use crate::Curve;
    
    #[test]
    fn test_de() {
        let curve: Curve = serde_json::
            from_str("\"30c:0%,40c:5%,50c:10%,60c:20%,70c:35%,80c:55%,90c:65%,100c:75%\"")
            .unwrap();
        
        assert_eq!(&curve.curve, &[
            30, 40, 50, 60,
            70, 80, 90, 100,
            0, 5, 10, 20,
            35, 55, 65, 75,
        ]);
    }
    
    #[test]
    fn test_ser() {
        let curve = Curve::
            from_config_str("30c:0%,40c:5%,50c:10%,60c:20%,70c:35%,80c:55%,90c:65%,100c:75%").unwrap();
        
        let json = serde_json::to_string(&curve).unwrap();
        
        assert_eq!(json, "\"30c:0%,40c:5%,50c:10%,60c:20%,70c:35%,80c:55%,90c:65%,100c:75%\"")
    }
}

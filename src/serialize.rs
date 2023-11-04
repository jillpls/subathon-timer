use serde_json::{Map, Value};
use subathon_timer::Error;

pub(crate) trait FastExtract {
    fn extract_object_key(&self, name: &str) -> Result<&Value, Error>;
    fn extract_object_str(&self, name: &str) -> Result<String, Error>;
    fn extract_object_obj(&self, name: &str) -> Result<&Map<String, Value>, Error>;
    fn extract_object_f64(&self, name: &str) -> Result<f64, Error>;
    fn extract_str(&self) -> Result<String, Error>;
    fn extract_obj(&self) -> Result<&Map<String, Value>, Error>;
    fn extract_f64(&self) -> Result<f64, Error>;
}

impl FastExtract for Value {
    fn extract_object_key(&self, name: &str) -> Result<&Value, Error> {
        self.as_object()
            .ok_or(Error::cne("obj"))?
            .get(name)
            .ok_or(Error::knf(name))
    }

    fn extract_object_str(&self, name: &str) -> Result<String, Error> {
        self.extract_object_key(name)?.extract_str()
    }

    fn extract_object_obj(&self, name: &str) -> Result<&Map<String, Value>, Error> {
        self.extract_object_key(name)?.extract_obj()
    }

    fn extract_object_f64(&self, name: &str) -> Result<f64, Error> {
        self.extract_object_key(name)?.extract_f64()
    }

    fn extract_str(&self) -> Result<String, Error> {
        Ok(self.as_str().ok_or(Error::cne("str"))?.to_string())
    }

    fn extract_obj(&self) -> Result<&Map<String, Value>, Error> {
        Ok(self.as_object().ok_or(Error::cne("obj"))?)
    }

    fn extract_f64(&self) -> Result<f64, Error> {
        Ok(self
            .as_number()
            .ok_or(Error::cne("num"))?
            .as_f64()
            .ok_or(Error::ftp("num", "f64"))?)
    }
}

use serde_json::{Map, Value};

pub(crate) trait FastExtract {
    fn extract_object_key(&self, name: &str) -> Result<&Value, String>;
    fn extract_object_str(&self, name: &str) -> Result<String, String>;
    fn extract_object_obj(&self, name: &str) -> Result<&Map<String, Value>, String>;
    fn extract_object_f64(&self, name: &str) -> Result<f64, String>;
    fn extract_str(&self) -> Result<String, String>;
    fn extract_obj(&self) -> Result<&Map<String, Value>, String>;
    fn extract_f64(&self) -> Result<f64, String>;
}

impl FastExtract for Value {
    fn extract_object_key(&self, name: &str) -> Result<&Value, String> {
        self.as_object()
            .ok_or("Not an object".to_string())?
            .get(name)
            .ok_or("Value not found".to_string())
    }

    fn extract_object_str(&self, name: &str) -> Result<String, String> {
        self.extract_object_key(name)?.extract_str()
    }

    fn extract_object_obj(&self, name: &str) -> Result<&Map<String, Value>, String> {
        self.extract_object_key(name)?.extract_obj()
    }

    fn extract_object_f64(&self, name: &str) -> Result<f64, String> {
        self.extract_object_key(name)?.extract_f64()
    }

    fn extract_str(&self) -> Result<String, String> {
        Ok(self.as_str().ok_or("Not a string".to_string())?
            .to_string())
    }

    fn extract_obj(&self) -> Result<&Map<String, Value>, String> {
        Ok(self.as_object().ok_or("Not an object".to_string())?)
    }

    fn extract_f64(&self) -> Result<f64, String> {
        Ok(self.as_number().ok_or("Not a string".to_string())?
            .as_f64().ok_or("Not a float")?)
    }
}

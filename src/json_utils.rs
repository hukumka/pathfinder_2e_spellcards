use anyhow::{anyhow, bail, Result};
use json::{object::Object, Array, JsonValue};

pub trait TypedParse: Sized {
    fn parse(object: &JsonValue) -> Result<Self>;
}

impl TypedParse for u8 {
    fn parse(object: &JsonValue) -> Result<Self> {
        object
            .as_u8()
            .ok_or_else(|| anyhow!("Wrong type: expected `u8`"))
    }
}

impl TypedParse for String {
    fn parse(object: &JsonValue) -> Result<Self> {
        object
            .as_str()
            .ok_or_else(|| anyhow!("Wrong type: expected `string`"))
            .map(|x| x.to_string())
    }
}

impl<T: TypedParse> TypedParse for Vec<T> {
    fn parse(object: &JsonValue) -> Result<Self> {
        let array = if let JsonValue::Array(array) = object {
            array
        } else {
            bail!("Wrong type: expected array")
        };
        array
            .iter()
            .enumerate()
            .map(|(i, obj)| {
                T::parse(obj)
                    .map_err(|err| err.context(format!("While parsing array at index `{i}`")))
            })
            .collect()
    }
}

pub trait ObjectExt {
    fn get_typed<T: TypedParse>(&self, key: &str) -> Result<T>;
    /// Same as get_typed, but element not being present is not an error.
    fn get_typed_maybe<T: TypedParse>(&self, key: &str) -> Result<Option<T>>;
}

impl ObjectExt for Object {
    fn get_typed<T: TypedParse>(&self, key: &str) -> Result<T> {
        let object = self
            .get(key)
            .ok_or_else(|| anyhow!("Field `{key}` missing"))?;
        Ok(T::parse(object).map_err(|e| e.context(format!("Then parsing field `{key}`")))?)
    }

    fn get_typed_maybe<T: TypedParse>(&self, key: &str) -> Result<Option<T>> {
        match self.get(key) {
            Some(object) => {
                let result = T::parse(object)
                    .map_err(|e| e.context(format!("Then parsing field `{key}`")))?;
                Ok(Some(result))
            }
            None => Ok(None),
        }
    }
}

pub trait JsonValueExt {
    fn as_array(&self) -> Result<&[JsonValue]>;
    fn as_object(&self) -> Result<&Object>;
}

impl JsonValueExt for JsonValue {
    fn as_array(&self) -> Result<&[JsonValue]> {
        if let JsonValue::Array(array) = self {
            Ok(array)
        } else {
            Err(anyhow!("Expected array."))
        }
    }

    fn as_object(&self) -> Result<&Object> {
        if let JsonValue::Object(object) = self {
            Ok(object)
        } else {
            Err(anyhow!("Expected array."))
        }
    }
}

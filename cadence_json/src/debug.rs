use std::fmt;

use crate::{AddressOwned, CompositeOwned};

impl fmt::Debug for CompositeOwned {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut map = f.debug_map();

        map.entry(&"id", &self.id);
        
        for field in &self.fields {
            map.entry(&field.name, &field.value);
        }

        map.finish()
    }
}

impl fmt::Debug for AddressOwned {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("0x")?;
        f.write_str(&hex::encode(&self.data))
    }
}
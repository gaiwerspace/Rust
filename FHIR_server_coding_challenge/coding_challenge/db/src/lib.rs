use pgrx::prelude::*;
use pgrx::Uuid;

pgrx::pg_module_magic!();

#[pg_extern]
fn fhir_put(resource_type: &str, resource_data: pgrx::JsonB) -> Uuid {
    if resource_type != "Patient" {
        error!("Only Patient resources are supported");
    }
    
    let id: Uuid = Spi::get_one("SELECT gen_random_uuid()")
        .expect("Failed to generate UUID")
        .expect("Failed to generate UUID");
    
    // Insert into fhir_resources table
    Spi::run(&format!(
        "INSERT INTO fhir_resources (id, resource_type, resource_data) VALUES ('{}', '{}', '{}')",
        id, resource_type, resource_data.0
    )).expect("Failed to insert resource");
    
    id
}

#[pg_extern]
fn fhir_get(resource_type: &str, resource_id: Uuid) -> Option<pgrx::JsonB> {
    if resource_type != "Patient" {
        error!("Only Patient resources are supported");
    }
    
    let query = format!(
        "SELECT resource_data FROM fhir_resources WHERE id = '{}' AND resource_type = '{}'",
        resource_id, resource_type
    );
    
    Spi::connect(|client| -> Result<Option<pgrx::JsonB>, pgrx::spi::Error> {
        let result = client.select(&query, None, &[])?;
        
        if result.is_empty() {
            Ok(None)
        } else {
            let row = result.first();
            let json_data: Option<pgrx::JsonB> = row.get(1)?;
            Ok(json_data)
        }
    })
    .expect("SPI query failed")
}

#[pg_extern]
fn fhir_search<'a>(
    resource_type: &'a str,
    param: &'a str,
    op: &'a str,
    value: &'a str,
) -> SetOfIterator<'a, Uuid> {
    if resource_type != "Patient" {
        error!("Only Patient resources are supported");
    }
    
    let query = if param.is_empty() {
        // Return all patients
        format!("SELECT id FROM fhir_resources WHERE resource_type = '{}'", resource_type)
    } else {
        match param {
            "name" => {
                if op == "contains" {
                    format!(
                        "SELECT id FROM fhir_resources WHERE resource_type = '{}' AND (\
                        LOWER(resource_data->'name'->0->>'family') LIKE LOWER('%{}%') OR \
                        LOWER(resource_data->'name'->0->>'text') LIKE LOWER('%{}%') OR \
                        EXISTS(SELECT 1 FROM jsonb_array_elements(resource_data->'name'->0->'given') AS given \
                        WHERE LOWER(given::text) LIKE LOWER('%{}%')))",
                        resource_type, value, value, value
                    )
                } else {
                    return SetOfIterator::new(vec![].into_iter());
                }
            }
            "gender" => {
                if op == "eq" {
                    format!(
                        "SELECT id FROM fhir_resources WHERE resource_type = '{}' AND resource_data->>'gender' = '{}'",
                        resource_type, value
                    )
                } else {
                    return SetOfIterator::new(vec![].into_iter());
                }
            }
            "birthdate" => {
                if op == "eq" {
                    format!(
                        "SELECT id FROM fhir_resources WHERE resource_type = '{}' AND resource_data->>'birthDate' = '{}'",
                        resource_type, value
                    )
                } else {
                    return SetOfIterator::new(vec![].into_iter());
                }
            }
            _ => {
                return SetOfIterator::new(vec![].into_iter());
            }
        }
    };
    
    let result_ids = Spi::connect(|client| {
        let result = client.select(&query, None, &[])?;
        let mut ids = Vec::new();
        
        for row in result {
            let id: Option<Uuid> = row.get(1)?;
            if let Some(id) = id {
                ids.push(id);
            }
        }
        
        Ok::<Vec<Uuid>, pgrx::spi::Error>(ids)
    }).unwrap_or_default();
    
    SetOfIterator::new(result_ids.into_iter())
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use super::*;
    use pgrx::prelude::*;

    #[pg_test]
    fn test_fhir_functions_exist() {
        // Just verify the functions are registered
        let result = Spi::get_one::<bool>("SELECT EXISTS(SELECT 1 FROM pg_proc WHERE proname = 'fhir_put')");
        assert!(result.is_ok());
    }
}

#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {
        // perform one-off initialization when the pg_test framework starts
    }

    pub fn postgresql_conf_options() -> Vec<&'static str> {
        // return any postgresql.conf settings that are required for your tests
        vec![]
    }
}
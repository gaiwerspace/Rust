use pgrx::prelude::*;
use serde_json::{json, JsonValue};
use uuid::Uuid;

pg_module_magic!();

#[pg_extern]
fn fhir_put(resource_type: &str, resource_data: pgrx::JsonB) -> String {
    if resource_type != "Patient" {
        error!("Only Patient resources are supported");
    }

    let mut client = pgrx::SPI::connect();
    let json_val = resource_data.0.clone();
    
    // Ensure the resource has required fields
    let mut patient = json_val.clone();
    if !patient.is_object() {
        error!("Resource data must be a JSON object");
    }
    
    // Generate ID if not present
    let id = if patient["id"].is_null() {
        Uuid::new_v4()
    } else {
        match Uuid::parse_str(patient["id"].as_str().unwrap_or("")) {
            Ok(uuid) => uuid,
            Err(_) => Uuid::new_v4(),
        }
    };
    
    patient["id"] = json!(id.to_string());
    patient["resourceType"] = json!("Patient");
    
    let query = format!(
        "INSERT INTO fhir_resources (id, resource_type, resource_data) \
         VALUES ('{}', $1, $2::jsonb) \
         ON CONFLICT (id) DO UPDATE SET resource_data = $2::jsonb, updated_at = CURRENT_TIMESTAMP",
        id
    );
    
    let prepared = client.prepare(&query, None).unwrap_or_else(|e| {
        error!("Failed to prepare statement: {}", e);
    });
    
    let _ = client.execute_with_args(
        &prepared,
        vec![
            (PgOid::BuiltinTypes::NAMEOID.oid(), resource_type.into_datum()),
            (PgOid::BuiltinTypes::JSONBOID.oid(), patient.to_string().into_datum()),
        ],
    ).unwrap_or_else(|e| {
        error!("Failed to execute insert: {}", e);
    });
    
    id.to_string()
}

#[pg_extern]
fn fhir_get(resource_type: &str, resource_id: &str) -> Option<pgrx::JsonB> {
    if resource_type != "Patient" {
        error!("Only Patient resources are supported");
    }

    let mut client = pgrx::SPI::connect();
    let query = "SELECT resource_data FROM fhir_resources \
                 WHERE resource_type = $1 AND id::text = $2 LIMIT 1";
    
    let prepared = client.prepare(query, None).unwrap_or_else(|e| {
        error!("Failed to prepare statement: {}", e);
    });
    
    let result = client.execute_with_args(
        &prepared,
        vec![
            (PgOid::BuiltinTypes::NAMEOID.oid(), resource_type.into_datum()),
            (PgOid::BuiltinTypes::TEXTOID.oid(), resource_id.into_datum()),
        ],
    ).unwrap_or_else(|e| {
        error!("Failed to execute select: {}", e);
    });
    
    if !result.is_empty() {
        result[0].get(0)
    } else {
        None
    }
}

#[pg_extern]
fn fhir_search(
    resource_type: &str,
    param: &str,
    op: &str,
    value: &str,
) -> SetOfIterator<'static, String> {
    if resource_type != "Patient" {
        error!("Only Patient resources are supported");
    }

    let mut client = pgrx::SPI::connect();
    let mut query = String::from(
        "SELECT id::text FROM fhir_resources WHERE resource_type = $1"
    );
    
    match param {
        "name" => {
            query.push_str(" AND (resource_data->'name' @> jsonb_build_array(jsonb_build_object('family', $2)) \
                           OR resource_data->'name' @> jsonb_build_array(jsonb_build_object('given', jsonb_build_array($2))))");
        }
        "gender" => {
            query.push_str(" AND resource_data->>'gender' = $2");
        }
        "birthDate" => {
            query.push_str(" AND resource_data->>'birthDate' = $2");
        }
        "_id" => {
            query.push_str(" AND id::text = $2");
        }
        _ => {
            error!("Unsupported search parameter: {}", param);
        }
    }
    
    let prepared = client.prepare(&query, None).unwrap_or_else(|e| {
        error!("Failed to prepare search statement: {}", e);
    });
    
    let result = client.execute_with_args(
        &prepared,
        vec![
            (PgOid::BuiltinTypes::NAMEOID.oid(), resource_type.into_datum()),
            (PgOid::BuiltinTypes::TEXTOID.oid(), value.into_datum()),
        ],
    ).unwrap_or_else(|e| {
        error!("Failed to execute search: {}", e);
    });
    
    let ids: Vec<String> = result
        .iter()
        .filter_map(|row| row.get(0))
        .collect();
    
    SetOfIterator::new(ids.into_iter())
}
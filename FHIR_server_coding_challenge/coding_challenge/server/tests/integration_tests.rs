use fhir_server::models::{Bundle, Patient};
use reqwest::Client;
use serde_json::json;

const BASE_URL: &str = "http://localhost:3000";

#[tokio::test]
async fn test_create_patient() {
    let client = Client::new();

    let patient = json!({
        "resourceType": "Patient",
        "name": [{
            "family": "Gauß",
            "given": ["Carl"],
            "text": "Carl Gauß"
        }],
        "gender": "male",
        "birthDate": "1990-01-01"
    });

    let response = client
        .post(format!("{}/fhir/Patient", BASE_URL))
        .header("Content-Type", "application/fhir+json")
        .json(&patient)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 201);

    let created_patient: Patient = response.json().await.expect("Failed to parse response");
    assert!(created_patient.id.is_some());
    assert_eq!(created_patient.resource_type, "Patient");
    assert!(created_patient.meta.is_some());
}

#[tokio::test]
async fn test_get_patient() {
    let client = Client::new();

    // First create a patient
    let patient = json!({
        "resourceType": "Patient",
        "name": [{
            "family": "Smith",
            "given": ["Jane"]
        }],
        "gender": "female",
        "birthDate": "1985-05-15"
    });

    let create_response = client
        .post(format!("{}/fhir/Patient", BASE_URL))
        .header("Content-Type", "application/fhir+json")
        .json(&patient)
        .send()
        .await
        .expect("Failed to create patient");

    let created_patient: Patient = create_response.json().await.expect("Failed to parse response");
    let patient_id = created_patient.id.unwrap();

    // Now get the patient
    let get_response = client
        .get(format!("{}/fhir/Patient/{}", BASE_URL, patient_id))
        .send()
        .await
        .expect("Failed to get patient");

    assert_eq!(get_response.status(), 200);

    let retrieved_patient: Patient = get_response.json().await.expect("Failed to parse response");
    assert_eq!(retrieved_patient.id.unwrap(), patient_id);
    assert_eq!(retrieved_patient.resource_type, "Patient");
}

#[tokio::test]
async fn test_search_patients() {
    let client = Client::new();

    // Create a few test patients
    let patients = vec![
        json!({
            "resourceType": "Patient",
            "name": [{
                "family": "Johnson",
                "given": ["Alice"]
            }],
            "gender": "female",
            "birthDate": "1992-03-20"
        }),
        json!({
            "resourceType": "Patient",
            "name": [{
                "family": "Brown",
                "given": ["Bob"]
            }],
            "gender": "male",
            "birthDate": "1988-07-10"
        }),
    ];

    for patient in patients {
        client
            .post(format!("{}/fhir/Patient", BASE_URL))
            .header("Content-Type", "application/fhir+json")
            .json(&patient)
            .send()
            .await
            .expect("Failed to create patient");
    }

    // Search by gender
    let search_response = client
        .get(format!("{}/fhir/Patient?gender=female", BASE_URL))
        .send()
        .await
        .expect("Failed to search patients");

    assert_eq!(search_response.status(), 200);

    let bundle: Bundle = search_response.json().await.expect("Failed to parse response");
    assert_eq!(bundle.resource_type, "Bundle");
    assert_eq!(bundle.bundle_type, "searchset");
    assert!(bundle.total > 0);
}

#[tokio::test]
async fn test_search_with_pagination() {
    let client = Client::new();

    let search_response = client
        .get(format!("{}/fhir/Patient?_count=5&_offset=0", BASE_URL))
        .send()
        .await
        .expect("Failed to search patients");

    assert_eq!(search_response.status(), 200);

    let bundle: Bundle = search_response.json().await.expect("Failed to parse response");
    assert_eq!(bundle.resource_type, "Bundle");
    assert!(bundle.entry.len() <= 5);
}

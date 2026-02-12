#!/bin/bash

BASE_URL="http://localhost:3000/fhir"

echo "FHIR Server API Demo"
echo "===================="
echo ""

# Create a patient
echo "1. Creating a patient..."
PATIENT_RESPONSE=$(curl -s -X POST "$BASE_URL/Patient" \
  -H "Content-Type: application/fhir+json" \
  -d '{
    "resourceType": "Patient",
    "name": [{
      "family": "Gauß",
      "given": ["Carl"],
      "text": "Carl Gauß"
    }],
    "gender": "male",
    "birthDate": "1990-01-01"
  }')

echo "Response: $PATIENT_RESPONSE"
echo ""

# Extract patient ID
PATIENT_ID=$(echo "$PATIENT_RESPONSE" | grep -o '"id":"[^"]*"' | cut -d'"' -f4)
echo "Created patient with ID: $PATIENT_ID"
echo ""

# Get the patient
echo "2. Retrieving the patient..."
curl -s "$BASE_URL/Patient/$PATIENT_ID" | jq '.'
echo ""

# Create another patient
echo "3. Creating another patient..."
curl -s -X POST "$BASE_URL/Patient" \
  -H "Content-Type: application/fhir+json" \
  -d '{
    "resourceType": "Patient",
    "name": [{
      "family": "Smith",
      "given": ["Jane"],
      "text": "Jane Smith"
    }],
    "gender": "female",
    "birthDate": "1985-05-15"
  }' | jq '.'
echo ""

# Search patients
echo "4. Searching patients by gender..."
curl -s "$BASE_URL/Patient?gender=male" | jq '.'
echo ""

echo "5. Searching patients by name..."
curl -s "$BASE_URL/Patient?name=doe" | jq '.'
echo ""

echo "6. Searching all patients with pagination..."
curl -s "$BASE_URL/Patient?_count=10&_offset=0" | jq '.'
echo ""

echo "Demo complete!"

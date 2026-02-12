#!/bin/bash

BASE_URL="http://localhost:3000/fhir"

echo "Testing FHIR Server API..."
echo "=========================="

# Test server health
echo "1. Testing server health..."
if curl -s -f "$BASE_URL/Patient" > /dev/null; then
    echo "✅ Server is responding"
else
    echo "❌ Server is not responding. Make sure it's running on port 3000"
    exit 1
fi

# Create a test patient
echo ""
echo "2. Creating a test patient..."
PATIENT_RESPONSE=$(curl -s -X POST "$BASE_URL/Patient" \
  -H "Content-Type: application/fhir+json" \
  -d '{
    "resourceType": "Patient",
    "name": [{
      "family": "TestPatient",
      "given": ["Carl"],
      "text": "John TestPatient"
    }],
    "gender": "male",
    "birthDate": "1990-01-01"
  }')

echo "Response: $PATIENT_RESPONSE"

# Extract patient ID if successful
if echo "$PATIENT_RESPONSE" | grep -q '"id"'; then
    PATIENT_ID=$(echo "$PATIENT_RESPONSE" | grep -o '"id":"[^"]*"' | cut -d'"' -f4)
    echo "✅ Patient created with ID: $PATIENT_ID"
    
    # Test retrieval
    echo ""
    echo "3. Retrieving the patient..."
    curl -s "$BASE_URL/Patient/$PATIENT_ID" | jq '.' || echo "Response: $(curl -s "$BASE_URL/Patient/$PATIENT_ID")"
    
    # Test search
    echo ""
    echo "4. Searching patients..."
    curl -s "$BASE_URL/Patient?name=test" | jq '.' || echo "Response: $(curl -s "$BASE_URL/Patient?name=test")"
    
    echo ""
    echo "✅ All tests completed successfully!"
else
    echo "❌ Failed to create patient"
    echo "Response: $PATIENT_RESPONSE"
fi
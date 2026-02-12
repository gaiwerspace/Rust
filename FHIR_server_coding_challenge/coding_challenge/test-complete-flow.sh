#!/bin/bash

BASE_URL="http://localhost:3000/fhir"

echo "🧪 Testing Complete FHIR Server Flow"
echo "===================================="
echo ""

# Test 1: Create a patient
echo "1️⃣ Creating a patient..."
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
if echo "$PATIENT_RESPONSE" | grep -q '"id"'; then
    PATIENT_ID=$(echo "$PATIENT_RESPONSE" | grep -o '"id":"[^"]*"' | cut -d'"' -f4)
    echo "✅ Patient created with ID: $PATIENT_ID"
    echo ""

    # Test 2: Retrieve the patient by ID
    echo "2️⃣ Retrieving patient by ID..."
    GET_RESPONSE=$(curl -s "$BASE_URL/Patient/$PATIENT_ID")
    echo "Response: $GET_RESPONSE"
    echo ""

    # Test 3: Create another patient for search testing
    echo "3️⃣ Creating another patient..."
    PATIENT2_RESPONSE=$(curl -s -X POST "$BASE_URL/Patient" \
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
      }')

    echo "Response: $PATIENT2_RESPONSE"
    echo ""

    # Test 4: Search by name
    echo "4️⃣ Searching patients by name 'doe'..."
    SEARCH_NAME_RESPONSE=$(curl -s "$BASE_URL/Patient?name=doe")
    echo "Response: $SEARCH_NAME_RESPONSE"
    echo ""

    # Test 5: Search by gender
    echo "5️⃣ Searching patients by gender 'male'..."
    SEARCH_GENDER_RESPONSE=$(curl -s "$BASE_URL/Patient?gender=male")
    echo "Response: $SEARCH_GENDER_RESPONSE"
    echo ""

    # Test 6: Search all patients
    echo "6️⃣ Searching all patients..."
    SEARCH_ALL_RESPONSE=$(curl -s "$BASE_URL/Patient")
    echo "Response: $SEARCH_ALL_RESPONSE"
    echo ""

    # Test 7: Search with pagination
    echo "7️⃣ Searching with pagination (_count=1)..."
    SEARCH_PAGINATED_RESPONSE=$(curl -s "$BASE_URL/Patient?_count=1")
    echo "Response: $SEARCH_PAGINATED_RESPONSE"
    echo ""

    echo "🎉 All tests completed!"

else
    echo "❌ Failed to create patient"
    echo "Response: $PATIENT_RESPONSE"

    # Check if server is running
    echo ""
    echo "🔍 Checking server status..."
    if curl -s -f "$BASE_URL/Patient" > /dev/null; then
        echo "✅ Server is responding"
    else
        echo "❌ Server is not responding. Make sure it's running on port 3000"
    fi
fi

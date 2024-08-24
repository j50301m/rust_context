#!/bin/bash

# Set your gRPC service details
PROTO_FILE="./example/api/protos/test.proto"
SERVICE_METHOD="test.TestService.TestMethod"
SERVICE_ADDRESS="localhost:12345"

# Set test parameters
TOTAL_REQUESTS=1000
CONCURRENCY=50

# Create a temporary file for the data template
TEMP_FILE=$(mktemp)

# Write the data template to the temporary file
cat << EOF > $TEMP_FILE
[
  {"test": "multi_thread_test_1"},
  {"test": "multi_thread_test_2"}
]
EOF

# Run the test
ghz --insecure \
    --proto $PROTO_FILE \
    --call $SERVICE_METHOD \
    --data-file $TEMP_FILE \
    -n $TOTAL_REQUESTS \
    -c $CONCURRENCY \
    $SERVICE_ADDRESS

# Clean up the temporary file
rm $TEMP_FILE
#!/bin/bash
mkdir ../uploads

cat - > "../uploads/$(date)"
echo "Content-Type: text/plain"
echo ""

echo "Method:         $REQUEST_METHOD"
echo "Content-Length: ${CONTENT_LENGTH:-unknown}"

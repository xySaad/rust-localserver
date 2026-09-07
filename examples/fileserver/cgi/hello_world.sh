#!/bin/bash

# HTTP Header (Content-Type + blank line delimiter)
echo "Content-Type: text/html; charset=utf-8"
echo ""

# HTML Body
cat <<EOF
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Bash CGI Script</title>
</head>
<body>
    <h1>Bash CGI Running</h1>

    <h3>Request Environment</h3>
    <ul>
        <li><strong>Request Method:</strong> ${REQUEST_METHOD:-N/A}</li>
        <li><strong>Query String:</strong> ${QUERY_STRING:-None}</li>
        <li><strong>Client IP:</strong> ${REMOTE_ADDR:-Unknown}</li>
        <li><strong>User Agent:</strong> ${HTTP_USER_AGENT:-Unknown}</li>
    </ul>
</body>
</html>
EOF

while true; do
    sleep 1s
    echo "<p><strong>Server Time:</strong> $(date)</p>"
done;
exit 0;
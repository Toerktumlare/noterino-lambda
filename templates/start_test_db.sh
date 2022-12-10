#!/bin/bash

set -e

echo "*******************************************"
echo "          | Starting DynamoDB |"
echo "          |  localhost:8080   |"
echo "*******************************************"
echo ""
docker stop dynamoDB
docker rm dynamoDB
docker run -d -p 8000:8000 --name dynamoDB amazon/dynamodb-local
docker ps -l
sleep 1
echo ""
echo "*******************************************"
echo "          | Container started |"
echo "          |  Importing data   |"
echo "*******************************************"
echo ""
aws dynamodb create-table --cli-input-yaml file://$PWD/template_local.yml --endpoint-url http://localhost:8000 | bat -l json -P
aws dynamodb batch-write-item --cli-input-yaml file://$PWD/mock_data.yml --endpoint-url http://localhost:8000 | bat -l json -P
aws dynamodb scan --table-name notes --endpoint-url http://localhost:8000 | bat -l json -P

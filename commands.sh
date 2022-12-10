aws dynamodb create-table --cli-input-json file://notes_table_import.json --endpoint-url http://localhost:8000

aws dynamodb create-table --generate-cli-skeleton

aws dynamodb describe-table --table-name notes > notes_table.json

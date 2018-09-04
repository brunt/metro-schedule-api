## Metro Schedule API
A simple web API to get the arrival time for a train given station and direction.

Request: POST localhost:8000/next-arrival
```json
{
	"station":"cwe",
	"direction":"west"
}
```

Response:
```json
{
	"station": "cwe",
	"direction": "west",
	"time": "9:44P"
}
```
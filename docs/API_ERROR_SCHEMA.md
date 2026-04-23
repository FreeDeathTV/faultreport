# FaultReport Error Envelope Schema

## RawError JSON Schema

Required for POST /api/projects/:id/errors

### Schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "required": ["message", "context"],
  "properties": {
    "message": {
      "type": "string",
      "maxLength": 1024
    },
    "stack": {
      "type": "string",
      "maxLength": 10485760
    },
    "context": {
      "type": "object",
      "required": ["url"],
      "properties": {
        "url": {
          "type": "string",
          "maxLength": 2048
        },
        "browser": { "type": "string" },
        "os": { "type": "string" },
        "user_id": { "type": "string" },
        "custom": {
          "type": "object",
          "maxLength": 5242880
        }
      }
    }
  }
}
```

### Examples

**Valid:**

```json
{
  "message": "Cannot read property 'x' of undefined",
  "stack": "at app.js:42",
  "context": {
    "url": "https://example.com/page"
  }
}
```

**Invalid (400):**

```json
{}  // Missing required
{
  "message": "a".repeat(2000)  // Too long
}
```

Validation on backend before hashing/storage.

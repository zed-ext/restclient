# HTTP Client Environments

This directory contains environment configurations for the REST Client extension.

## environments.json

Define different environments (development, staging, production) with their specific variables.

**Format:**
```json
{
  "environment_name": {
    "variable_name": "variable_value",
    "another_var": "another_value"
  }
}
```

**Usage in .http files:**
```http
GET {{base_url}}/api/users
Authorization: Bearer {{auth_token}}
```

## Variable Resolution Order

Variables are resolved in the following order (highest priority first):

1. **File-level variables** - Defined in .http file with `@variable = value`
2. **Global variables** - Set by response handlers with `client.global.set()`
3. **Environment variables** - From the active environment in environments.json
4. **System environment variables** - From your operating system

## Switching Environments

Use the Zed command palette:
- `REST Client: Set Environment` → Choose from available environments

## Security Notes

- **DO NOT** commit sensitive credentials to version control
- Use system environment variables for secrets: `{{SECRET_API_KEY}}`
- Or use a `.http-client/environments.local.json` file (add to .gitignore)

## Example Workflow

1. Create environments.json with your API endpoints
2. Set active environment (e.g., "development")
3. Use variables in your .http files: `GET {{api_url}}/users`
4. Switch environments without changing request files
